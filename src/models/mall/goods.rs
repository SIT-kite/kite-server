use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::mall::{MallError, NewGoods, CoverInfo, DetailInfo, Publish, UpdateGoods, SelectGoods};
use crate::models::PageView;

use super::{GoodsDetail, SimpleGoods};
use chrono::{DateTime, Utc, Local};
use serde_json::Value;
use std::borrow::Borrow;
use rand::Rng;

pub async fn get_goods_list(db: &PgPool, form: &SelectGoods) -> Result<Vec<CoverInfo>> {

    let sort = match &form.sort{
        Some(value) => value,
        None => &0
    };

    let item_name = match &form.item_name {
        Some(value) => format!("%{}%",value),
        None => "".to_string()
    };

    let page = match &form.page {
        Some(value) => value,
        None => &0
    };

    let goods = sqlx::query_as(
        "
            SELECT
                A.pub_code
                ,A.item_code
                ,A.views
                ,A1.item_name
                ,A1.price
                ,A1.cover_image
            FROM mall.publish A
            LEFT JOIN mall.commodity A1
                   ON A.item_code = A1.item_code
            WHERE A.status = 'Y'
                  AND ($1 =  0 OR $1 <>  0 AND A1.sort = $1)
                  AND ($2 = '' OR $2 <> '' AND A1.item_name LIKE $2)
            ORDER BY A.insert_time
            LIMIT $3 OFFSET $4;
            "
    )

        .bind(sort)
        .bind(item_name)
        .bind(10)           //每次取10个
        .bind(page * 10)    //从page*10 开始取
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

pub async fn get_goods_detail(db: &PgPool, goods_id: String) -> Result<Vec<DetailInfo>> {
    let goods = sqlx::query_as(
        "SELECT
                item_name
                ,description
                ,price
                ,images
            FROM mall.commodity
            WHERE item_code = $1;",
    )
        .bind(goods_id)
        .fetch_all(db)
        .await?;

    Ok(goods)
}

pub async fn delete_goods(db: &PgPool, pub_code: String) -> Result<i32> {
    let delete:Option<(i32,)>  = sqlx::query_as(
        "
            UPDATE mall.publish
            SET
                status = 'N'
                ,update_time = $1
            WHERE
                pub_code = $2;
            "
    )
        .bind(Local::now())
        .bind(pub_code)
        .fetch_optional(db)
        .await?;
    Ok(1)
}

pub async fn publish_goods(db: &PgPool, uid: i32, new: &Publish) -> Result<String> {
    //获取当前时间作为编号
    let mut rng = rand::thread_rng();
    let utc: DateTime<Utc> = Utc::now();
    let code  = utc.format("%Y%m%d%S").to_string();

    //编号头+年月日秒+随机三位数构成编号
    let pub_code = format!("P{}{}",code,rng.gen_range(100,999));
    let item_code = format!("G{}{}",code,rng.gen_range(100,999));

    let insert_publish:Option<(i32,)>  = sqlx::query_as(
        "
            INSERT INTO mall.publish(
                 pub_code
                ,publisher
                ,item_code
                ,campus
                ,status
                ,views
                ,insert_time
                ,update_time
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8);
        ",
    )
        .bind(&pub_code)
        .bind(uid)
        .bind(&item_code)
        .bind(&new.campus)
        .bind("Y")
        .bind(0)
        .bind(Local::now())
        .bind(Local::now())
        .fetch_optional(db)
        .await?;

    let insert_commodity:Option<(i32,)>  = sqlx::query_as(
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);"
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

pub async fn update_goods(db: &PgPool, new: &UpdateGoods) -> Result<i32> {
    let returning: Option<(i32,)> = sqlx::query_as(
        "
            UPDATE
                mall.commodity
             SET
                item_name = $1
                ,description = $2
                ,price = $3
                ,images = $4
                ,cover_image = $5
                ,sort = $6
                ,update_time = $7
             WHERE
                item_code = $8
             "
    )
        .bind(&new.item_name)
        .bind(&new.description)
        .bind(&new.price)
        .bind(&new.images)
        .bind(&new.cover_image)
        .bind(&new.sort)
        .bind(Local::now())
        .bind(&new.item_code)
        .fetch_optional(db)
        .await?;

    Ok(1)
}

pub async fn update_views(db: &PgPool, pub_code: String) -> Result<i32> {
    //调用更新views的存储过程
    let update:Option<(i32,)>  = sqlx::query_as(
        "
                select update_views($1)
            "
    )
        .bind(pub_code)
        .fetch_optional(db)
        .await?;
    Ok(1)
}
