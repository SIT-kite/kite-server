use sqlx::PgPool;
use serde_json::Value;
use crate::error::Result;
use chrono::Local;
use crate::models::mall::{Wish};


pub async fn insert_wish(db: &PgPool, uid: i32, new: &Value) -> Result<i32> {

    let insert_publish:Option<(i32,)>  = sqlx::query_as(
        "
            INSERT INTO mall.wish(
                user_code
                ,pub_code
                ,insert_time
                ,update_time
            )
            VALUES ($1,$2,$3,$4);
        "
    )
        .bind(uid)
        .bind(&new["pub_code"].as_str().unwrap())
        .bind(Local::now())
        .bind(Local::now())
        .fetch_optional(db)
        .await?;

    Ok(1)
}

pub async fn cancel_wish(db: &PgPool, uid: i32, pub_code: String) -> Result<()> {

    let returning: Option<(i32,)> =  sqlx::query_as(
        "
            DELETE FROM mall.wish
            WHERE user_code = $1
                AND pub_code = $2
             "
    )
        .bind(uid)
        .bind(pub_code)
        .fetch_optional(db)
        .await?;

    Ok(())
}

pub async fn get_wishes(db: &PgPool, user_code: i32) -> Result<Vec<Wish>> {
    let goods = sqlx::query_as(
        "
            SELECT
                A.pub_code
                ,B.item_code
                ,B.views
                ,B.status
                ,C.item_name
                ,C.price
                ,C.cover_image
            FROM mall.wish A
            LEFT JOIN mall.publish B
                   ON A.pub_code = B.pub_code
            LEFT JOIN mall.commodity C
                   ON B.item_code = C.item_code
            WHERE A.user_code = $1
            "
    )
        .bind(user_code)
        .fetch_all(db)
        .await?;

    Ok(goods)
}