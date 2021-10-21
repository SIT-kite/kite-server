use chrono::{DateTime, Local};
use rand::Rng;
use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::mall::{CoverInfo, DetailInfo, MallError, Publish, SelectGoods, UpdateGoods};
use crate::models::PageView;

use super::SimpleGoods;
use wechat_sdk::wechat::CheckResult;

pub async fn get_goods_list(db: &PgPool, form: &SelectGoods, page: PageView) -> Result<Vec<CoverInfo>> {
    let like_clause = format!("%{}%", form.keyword);

    let goods = sqlx::query_as(
        "
                SELECT
                    A.pub_code
                    ,A.item_code
                    ,COALESCE(B.views,0) AS views
                    ,A1.item_name
                    ,A1.price
                    ,A1.cover_image
                FROM mall.publish A
                LEFT JOIN mall.commodity A1
                        ON A.item_code = A1.item_code
                LEFT JOIN (
                        SELECT
                            item_code,
                            count(item_code) AS views
                        FROM mall.views
                        GROUP BY item_code
                    ) AS B
                        ON A.item_code = B.item_code
                LEFT JOIN mall.check C
                        ON A.check_code = C.check_code
                WHERE A.status = 'Y'
                  AND C.label = '100'
                  AND ($1 IS NULL OR A1.sort = $1)
                  AND ($2 IS NULL OR A1.item_name LIKE $2)
                ORDER BY A.insert_time
                LIMIT $3 OFFSET $4;
            ",
    )
        .bind(&form.sort)
        .bind(like_clause)
        .bind(page.count(10u16) as i32) // 每次最大取 10 个
        .bind(page.index() as i32)
        .fetch_all(db)
        .await?;

    Ok(goods)
}

pub async fn query_goods(
    db: &PgPool,
    keyword: &str,
    sort: i32,
    page: PageView,
) -> Result<Vec<SimpleGoods>> {
    let goods = sqlx::query_as(
        "SELECT id, title, cover_image, tags, price, status, publish_time
        FROM mall.query_goods($1, $2)
        ORDER BY publish_time DESC
        LIMIT $3 OFFSET $4;",
    )
        .bind(keyword)
        .bind(sort)
        .bind(page.count(20) as i16)
        .bind(page.offset(20) as i16)
        .fetch_all(db)
        .await?;

    Ok(goods)
}

pub async fn get_goods_detail(db: &PgPool, item_code: &String) -> Result<DetailInfo> {
    //获取商品详情
    let detail = sqlx::query_as(
        "SELECT
                item_name
                ,description
                ,price
                ,images
            FROM mall.commodity
            WHERE item_code = $1
            LIMIT 1;",
    )
        .bind(item_code)
        .fetch_optional(db)
        .await?;

    detail.ok_or_else(|| ApiError::new(MallError::NoSuchGoods))
}

pub async fn delete_goods(db: &PgPool, pub_code: &String) -> Result<i32> {
    let _ = sqlx::query(
        "
            UPDATE mall.publish
            SET
                status = 'N'
                ,update_time = $1
            WHERE
                pub_code = $2;
            ",
    )
        .bind(Local::now())
        .bind(pub_code)
        .fetch_optional(db)
        .await?;
    Ok(1)
}

pub async fn publish_goods(db: &PgPool, uid: i32, new: &Publish) -> Result<String> {
    // 获取当前时间作为编号
    let mut rng = rand::thread_rng();
    let current_time: DateTime<Local> = Local::now();
    let code = current_time.format("%Y%m%d%S").to_string();

    //编号头+年月日秒+随机三位数构成编号
    let pub_code = format!("P{}{}", code, rng.gen_range(100, 999));
    let item_code = format!("G{}{}", code, rng.gen_range(100, 999));

    let _ = sqlx::query(
        "
            INSERT INTO mall.publish(
                 pub_code
                ,publisher
                ,item_code
                ,campus
                ,status
                ,insert_time
                ,update_time
                ,check_code
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8);
        ",
    )
        .bind(&pub_code)
        .bind(uid)
        .bind(&item_code)
        .bind(&new.campus)
        .bind("Y")
        .bind(Local::now())
        .bind(Local::now())
        .bind(&new.check_code)
        .fetch_optional(db)
        .await?;

    let _ = sqlx::query(
        "
            INSERT INTO mall.commodity(
                 item_code
                ,item_name
                ,description
                ,price
                ,images
                ,cover_image
                ,sort
                ,insert_time
                ,update_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);",
    )
        .bind(&item_code)
        .bind(&new.item_name)
        .bind(&new.description)
        .bind(&new.price)
        .bind(&new.images)
        .bind(&new.cover_image)
        .bind(&new.sort)
        .bind(Local::now())
        .bind(Local::now())
        .fetch_optional(db)
        .await?;

    Ok(item_code)
}

pub async fn check_msg_save(db: &PgPool, new: &CheckResult) -> Result<String> {
    // 获取当前时间作为编号
    let mut rng = rand::thread_rng();
    let current_time: DateTime<Local> = Local::now();
    let code = current_time.format("%Y%m%d%S").to_string();

    //编号头+年月日秒+随机三位数构成编号
    let check_code = format!("K{}{}", code, rng.gen_range(100, 999));

    let mut Detail:String = "".to_string();

    //将数组中[{},{}] 转换为 '{}','{}' 字符串
    for d in &new.detail {
        Detail = format!("{}'{}',",Detail,serde_json::json!(d));
    }

    //弹出末尾多余的','
    Detail.pop();

    let _ = sqlx::query(
        "
            INSERT INTO mall.check(
                check_code
                ,errcode
                ,errmsg
                ,detail
                ,suggest
                ,label
                ,trace_id
                ,insert_time
                ,update_time
            )
            VALUES ($1,$2,$3,Array[$4],$5,$6,$7,$8,$9);
        ",
    )
        .bind(&check_code)
        .bind(&new.errcode)
        .bind(&new.errmsg)
        .bind(Detail)
        .bind(&new.result.suggest)
        .bind(&new.result.label)
        .bind(&new.trace_id)
        .bind(Local::now())
        .bind(Local::now())
        .fetch_optional(db)
        .await?;

    Ok(check_code)
}

pub async fn check_goods(db: &PgPool, uid: i32, new: &UpdateGoods) -> Result<String> {
    let item_code: Option<(String,)> = sqlx::query_as(
        "
            SELECT item_code
            FROM mall.publish
            WHERE publisher = $1
              AND pub_code = $2
              AND status = 'Y'
            LIMIT 1
        ",
    )
        .bind(uid)
        .bind(&new.pub_code)
        .fetch_optional(db)
        .await?;

    item_code
        .map(|(item_code,)| item_code)
        .ok_or_else(|| ApiError::new(MallError::NoUserGood))
}

pub async fn update_views(db: &PgPool, pub_code: String) -> Result<()> {
    //调用更新views的存储过程
    let _ = sqlx::query(
        "
                SELECT update_views($1)
            ",
    )
        .bind(pub_code)
        .fetch_one(db)
        .await?;
    Ok(())
}

pub async fn insert_view_log(db: &PgPool, uid: i32, item_code: &String) -> Result<()> {
    let _ = sqlx::query(
        "
               INSERT INTO mall.views(
                    user_code,
                    item_code,
                    view_time)
               VALUES($1,$2,$3);
            ",
    )
        .bind(uid)
        .bind(item_code)
        .bind(Local::now())
        .execute(db)
        .await?;

    Ok(())
}
