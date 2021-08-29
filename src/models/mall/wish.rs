use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::mall::{MallError, Wish};

pub async fn insert_wish(db: &PgPool, uid: i32, pub_code: String) -> Result<()> {
    let _ = sqlx::query(
        "
            INSERT INTO mall.wish(
                user_code
                ,pub_code
                ,insert_time
                ,update_time
            )
            VALUES ($1, $2, now(), now());
        ",
    )
    .bind(uid)
    .bind(pub_code)
    .fetch_optional(db)
    .await?;

    Ok(())
}

pub async fn cancel_wish(db: &PgPool, uid: i32, pub_code: String) -> Result<()> {
    let _ = sqlx::query(
        "
            DELETE FROM mall.wish
            WHERE user_code = $1
                AND pub_code = $2
             ",
    )
    .bind(uid)
    .bind(pub_code)
    .fetch_optional(db)
    .await?;

    Ok(())
}

pub async fn get_user_wishes(db: &PgPool, user_code: i32) -> Result<Vec<Wish>> {
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
            ",
    )
    .bind(user_code)
    .fetch_all(db)
    .await?;

    if !goods.is_empty() {
        Ok(goods)
    } else {
        Err(ApiError::new(MallError::NoWish))
    }
}
