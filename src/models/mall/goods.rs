use sqlx::PgPool;

use crate::error::Result;
use crate::models::mall::{CoverInfo, DetailInfo};
use crate::models::PageView;

use super::SimpleGoods;
use chrono::{DateTime, Local, Utc};
use rand::Rng;
use serde_json::Value;

pub async fn get_goods_list(db: &PgPool, form: &Value) -> Result<Vec<CoverInfo>> {
    let sort;
    let item_name;
    let page;

    //判断是否有传来的值
    if form["sort"].is_null() {
        sort = 0;
    } else {
        sort = form["sort"].as_i64().unwrap() as i32;
    }

    if form["item_name"].is_null() {
        item_name = "".to_string();
    } else {
        item_name = format!("%{}%", form["item_name"].as_str().unwrap());
    }

    //每次提取10个数据未传值则page 为0
    if form["page"].is_null() {
        page = 0
    } else {
        page = form["page"].as_i64().unwrap() as i32;
    }

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
            ",
    )
    .bind(sort)
    .bind(item_name)
    .bind(10) //每次取10个
    .bind(page * 10) //从page*10 开始取
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
    let delete: Option<(i32,)> = sqlx::query_as(
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

pub async fn publish_goods(db: &PgPool, uid: i32, new: &Value) -> Result<i32> {
    //获取当前时间作为编号
    let mut rng = rand::thread_rng();
    let utc: DateTime<Utc> = Utc::now();
    let code = utc.format("%Y%m%d%S").to_string();

    //编号头+年月日秒+随机三位数构成编号
    let pub_code = format!("P{}{}", code, rng.gen_range(100, 999));
    let item_code = format!("G{}{}", code, rng.gen_range(100, 999));

    /*
        //根据key 提取Json 的Value时会导致存在”“
        println!("{}",new["item_name"]);
        //将提取值利用as_str转化为&str 后拆包即可得到无”“的值
        println!("{}",new["item_name"].as_str().unwrap());
    */

    let insert_publish: Option<(i32,)> = sqlx::query_as(
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
    .bind(pub_code)
    .bind(uid)
    .bind(&item_code)
    .bind(&new["campus"].as_str().unwrap())
    .bind("Y")
    .bind(0)
    .bind(Local::now())
    .bind(Local::now())
    .fetch_optional(db)
    .await?;

    let insert_commodity: Option<(i32,)> = sqlx::query_as(
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
    .bind(&new["item_name"].as_str().unwrap())
    .bind(&new["description"].as_str().unwrap())
    .bind(&new["price"].as_f64().unwrap())
    .bind(&new["image"].as_str().unwrap())
    .bind(&new["cover_image"].as_str().unwrap())
    .bind(&new["sort"].as_i64().unwrap())
    .bind(Local::now())
    .bind(Local::now())
    .fetch_optional(db)
    .await?;

    Ok(1)
}

pub async fn update_goods(db: &PgPool, new: &Value) -> Result<i32> {
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
             ",
    )
    .bind(&new["item_name"].as_str().unwrap())
    .bind(&new["description"].as_str().unwrap())
    .bind(&new["price"].as_f64().unwrap())
    .bind(&new["images"].as_str().unwrap())
    .bind(&new["cover_image"].as_str().unwrap())
    .bind(&new["sort"].as_i64().unwrap())
    .bind(Local::now())
    .bind(&new["item_code"].as_str().unwrap())
    .fetch_optional(db)
    .await?;

    Ok(1)
}

pub async fn update_views(db: &PgPool, pub_code: String) -> Result<i32> {
    //调用更新views的存储过程
    let update: Option<(i32,)> = sqlx::query_as(
        "
                select update_views($1)
            ",
    )
    .bind(pub_code)
    .fetch_optional(db)
    .await?;
    Ok(1)
}
