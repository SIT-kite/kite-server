use chrono::{DateTime, Local, Utc};
use rand::Rng;
use sqlx::PgPool;

use crate::error::Result;
use crate::models::mall::{Comment, PubComment};

pub async fn publish_comment(db: &PgPool, uid: i32, new: &PubComment) -> Result<String> {
    // 随机数
    let mut rng = rand::thread_rng();
    // 获取当前时间作为编号
    let utc: DateTime<Utc> = Utc::now();
    let code = utc.format("%Y%m%d%S").to_string();

    // 编号头+年月日秒+随机三位数构成编号
    let com_code = format!("C{}{}", code, rng.gen_range(100, 999));

    let parent_code = match &new.parent_code {
        Some(value) => value,
        None => "",
    };

    let _ = sqlx::query(
        "
            INSERT INTO mall.comment(
                 com_code
                ,user_code
                ,item_code
                ,content
                ,parent_code
                ,num_like
                ,status
                ,insert_time
                ,update_time
                ,check_code
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10);
        ",
    )
    .bind(&com_code)
    .bind(uid)
    .bind(&new.item_code)
    .bind(&new.content)
    .bind(parent_code)
    .bind(0)
    .bind("Y")
    .bind(Local::now())
    .bind(Local::now())
    .bind(&new.check_code)
    .fetch_optional(db)
    .await?;

    Ok(com_code)
}

pub async fn delete_comment(db: &PgPool, com_code: String) -> Result<()> {
    let _ = sqlx::query(
        "
            UPDATE
                mall.comment
             SET
                status = 'N'
                ,update_time = $1
             WHERE com_code = $2
                OR parent_code = $2
             ",
    )
    .bind(Local::now())
    .bind(com_code)
    .fetch_optional(db)
    .await?;

    Ok(())
}

pub async fn get_comments(db: &PgPool, item_code: String) -> Result<Vec<Comment>> {
    let goods = sqlx::query_as(
        "
            SELECT
                com_code
                ,user_code
                ,content
                ,parent_code
                ,num_like
            FROM mall.comment A
            LEFT JOIN mall.check B
                    ON A.check_code = B.check_code
            WHERE status = 'Y'
                AND B.label = '100'
                AND item_code = $1;
            ",
    )
    .bind(item_code)
    .fetch_all(db)
    .await?;

    Ok(goods)
}

pub async fn update_num_like(db: &PgPool, com_code: String) -> Result<()> {
    // 调用更新num_like的存储过程
    let _ = sqlx::query(
        "
                SELECT update_num_like($1);
            ",
    )
    .bind(com_code)
    .fetch_one(db)
    .await?;

    Ok(())
}
