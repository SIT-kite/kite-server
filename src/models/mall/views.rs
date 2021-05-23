use sqlx::PgPool;

use crate::error::Result;

use super::Views;

pub async fn get_views(db: &PgPool, goods: i32) -> Result<Vec<Views>> {
    let comments: Vec<_> =
        sqlx::query_as("SELECT person, goods, ts FROM mall.views WHERE goods = $1 ORDER BY ts DESC;")
            .bind(goods)
            .fetch_all(db)
            .await?;
    Ok(comments)
}

pub async fn append_views(db: &PgPool, uid: i32, goods_id: i32) -> Result<()> {
    sqlx::query("INSERT INTO mall.Views (person, goods) VALUES ($1, $2);")
        .bind(uid)
        .bind(goods_id)
        .execute(db)
        .await?;
    Ok(())
}
