use sqlx::PgPool;
use tokio::io::AsyncWriteExt;

use crate::bridge::{
    Activity, ActivityDetail, ActivityDetailRequest, ActivityListRequest, AgentManager, HostError,
    RequestFrame, RequestPayload, ResponsePayload, SaveScActivity, SaveScScore, ScActivityItem,
    ScActivityRequest, ScDetail, ScImages, ScScoreItem, ScScoreItemRequest,
};
use crate::config::CONFIG;
use crate::error::{ApiError, Result};
use crate::models::file::AvatarImage;

static URL_PREFIX: &str = "https://kite.sunnysab.cn/sc/image/";

pub async fn query_current_sc_score_list(
    agent: &AgentManager,
    data: ScScoreItemRequest,
) -> Result<Vec<ScScoreItem>> {
    let request = RequestFrame::new(RequestPayload::ScScoreDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScScoreDetail(sc_score) = response {
        Ok(sc_score)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_score_list(db: &PgPool, data: Vec<SaveScScore>) -> Result<()> {
    for each_score in data {
        sqlx::query(
            "INSERT INTO events.sc_score_detail (student_id, activity_id, amount)
            VALUES ($1, $2, $3)
            ON CONFLICT (student_id, activity_id, amount) DO NOTHING;",
        )
        .bind(each_score.account)
        .bind(each_score.activity_id)
        .bind(each_score.amount)
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn query_current_sc_activity_list(
    agent: &AgentManager,
    data: ScActivityRequest,
) -> Result<Vec<ScActivityItem>> {
    let request = RequestFrame::new(RequestPayload::ScActivityDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ScActivityDetail(sc_activity) = response {
        Ok(sc_activity)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_sc_activity_list(db: &PgPool, data: Vec<SaveScActivity>) -> Result<()> {
    for each_activity in data {
        sqlx::query(
            "INSERT INTO events.sc_activity_detail (student_id, activity_id, time, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (student_id, activity_id, time) DO NOTHING;",
        )
        .bind(each_activity.account)
        .bind(each_activity.activity_id)
        .bind(each_activity.time)
        .bind(each_activity.status)
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn get_sc_score_detail(pool: &PgPool, query: &str) -> Result<Vec<ScDetail>> {
    let result = sqlx::query_as(
        "SELECT detail.activity_id, event.title, time, status, amount, category
        FROM events.sc_detail as detail, events.sc_events as event
        WHERE detail.activity_id = event.activity_id and student_id = $1;",
    )
    .bind(query)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

pub async fn query_activity_list(
    agent: &AgentManager,
    data: ActivityListRequest,
) -> Result<Vec<Activity>> {
    let request = RequestFrame::new(RequestPayload::ActivityList(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ActivityList(list) = response {
        Ok(list)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn query_activity_detail(
    agent: &AgentManager,
    data: ActivityDetailRequest,
) -> Result<Box<ActivityDetail>> {
    let request = RequestFrame::new(RequestPayload::ActivityDetail(data));
    let response = agent.request(request).await??;
    if let ResponsePayload::ActivityDetail(detail) = response {
        Ok(detail)
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

pub async fn save_image_as_file(data: &Vec<ScImages>) -> Result<(Vec<AvatarImage>, Vec<String>)> {
    let mut result = Vec::new();
    let mut image_uuid = Vec::new();
    for image in data {
        let path = format!("{}/image/{}", &CONFIG.server.attachment, image.new_name);
        let mut file = tokio::fs::File::create(&path).await?;
        file.write_all(&image.content).await?;
        let (file_name, _) = image.new_name.split_once(".").unwrap_or_default();
        image_uuid.push(file_name.to_string());
        let image_uuid = file_name.parse().unwrap();
        let size = image.content.len();
        let avatar =
            AvatarImage::with_id(image_uuid)
                .set_uploader(0)
                .set_file(URL_PREFIX, path, size as i32);
        result.push(avatar);
    }

    Ok((result, image_uuid))
}

pub async fn save_image(db: &PgPool, data: Vec<AvatarImage>) -> Result<()> {
    for each_image in data {
        sqlx::query(
            "INSERT INTO public.attachments (id, path, uploader, upload_time, size, url)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id",
        )
        .bind(each_image.id)
        .bind(&each_image.path)
        .bind(&each_image.uploader)
        .bind(&each_image.upload_time)
        .bind(&each_image.size)
        .bind(&each_image.url)
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn save_sc_activity_detail(
    db: &PgPool,
    data: &ActivityDetail,
    image_uuid: Vec<String>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO events.sc_events (activity_id, category, title, start_time, sign_start_time, sign_end_time, place, duration, manager, contact, organizer, undertaker, description, image)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (activity_id) DO NOTHING;"
    )
        .bind(data.id)
        .bind(data.category)
        .bind(&data.title)
        .bind(data.start_time)
        .bind(data.sign_start_time)
        .bind(data.sign_end_time)
        .bind(&data.place)
        .bind(&data.duration)
        .bind(&data.manager)
        .bind(&data.contact)
        .bind(&data.organizer)
        .bind(&data.undertaker)
        .bind(&data.description)
        .bind(image_uuid)
        .execute(db)
        .await?;

    Ok(())
}

async fn update_activity_list_in_category(
    pool: PgPool,
    agents: AgentManager,
    category: i32,
) -> Result<()> {
    let id: Option<(i32,)> = sqlx::query_as(
        "SELECT activity_id FROM events.sc_events 
        WHERE category = $1 ORDER BY activity_id DESC LIMIT 1;",
    )
    .bind(category)
    .fetch_optional(&pool)
    .await?;

    let id = id.map(|x| x.0).unwrap_or_default();
    let page_count = 50;
    let mut last_index = 1;

    loop {
        let param = ActivityListRequest {
            count: page_count,
            index: last_index,
            category,
        };
        let activity_list = query_activity_list(&agents, param).await?;
        let fetched_size = activity_list.len();

        update_activity_in_category(id, &pool, activity_list, &agents).await?;

        if fetched_size < page_count as usize {
            break;
        }
        last_index += 1;
    }
    Ok(())
}

async fn update_activity_in_category(
    id: i32,
    pool: &PgPool,
    activity_list: Vec<Activity>,
    agents: &AgentManager,
) -> Result<()> {
    for each_activity in activity_list {
        if id == each_activity.id {
            break;
        }
        let data = ActivityDetailRequest { id: each_activity.id };

        let mut activity_detail = query_activity_detail(agents, data).await;
        let mut detail;
        match activity_detail {
            Ok(d) => {
                detail = d;
                detail.category = each_activity.category;

                // Save the image
                let (image_message, image_uuid) = save_image_as_file(&detail.images).await?;

                save_image(&pool, image_message).await?;
                save_sc_activity_detail(&pool, detail.as_ref(), image_uuid).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn update_activity_list(pool: &PgPool, agents: &AgentManager) -> Result<()> {
    let mut handlers = vec![];

    // todo: 1~11
    for i in 3..=3 {
        let handle = tokio::spawn(update_activity_list_in_category(pool.clone(), agents.clone(), i));
        handlers.push(handle);
    }

    // TODO: Wait handles. Must be updated.
    for h in handlers {
        let r = h.await;
        println!("{:?}", r);
    }

    Ok(())
}

pub async fn activity_update_daemon(pool: PgPool, agents: AgentManager) -> Result<()> {
    // loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
    update_activity_list(&pool, &agents).await?;
    // }
    Ok(())
}
