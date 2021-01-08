use super::Favorites;
use crate::error::{ApiError, Result};
use sqlx::PgPool;

pub async fn get_favorites(db: &PgPool, person: i32) -> Result<Vec<Favorites>> {
    let comments: Vec<_> = sqlx::query_as!(
        Favorites,
        "SELECT f.goods, g.title, g.cover_image AS image, f.ts
        FROM mall.favorites f, mall.goods g, public.person p
        WHERE f.person = p.uid AND p.uid = $1 AND f.goods = g.id
        ORDER BY ts DESC;",
        person
    )
    .fetch_all(db)
    .await?;
    Ok(comments)
}

pub async fn append_favorites(db: &PgPool, uid: i32, goods_id: i32) -> Result<()> {
    sqlx::query!(
        "INSERT INTO mall.favorites (person, goods)
        VALUES ($1, $2)
        ON CONFLICT (person, goods) DO NOTHING;",
        uid,
        goods_id,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_favorites(db: &PgPool, uid: i32, goods_id: i32) -> Result<()> {
    sqlx::query!(
        "DELETE FROM mall.favorites WHERE person = $1 AND goods = $2;",
        uid,
        goods_id
    )
    .execute(db)
    .await?;
    Ok(())
}
