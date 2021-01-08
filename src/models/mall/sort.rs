use crate::error::Result;
use crate::models::mall::Sorts;
use sqlx::PgPool;

pub async fn get_goods_sorts(db: &PgPool) -> Result<Vec<Sorts>> {
    let sorts = sqlx::query_as!(Sorts, "SELECT id, title FROM mall.sorts ORDER BY priority;")
        .fetch_all(db)
        .await?;
    Ok(sorts)
}
