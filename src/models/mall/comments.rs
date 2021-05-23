use crate::error::Result;
use crate::models::mall::{GoodsComment, NewComment};
use sqlx::PgPool;

pub async fn get_comments(db: &PgPool, goods_id: i32) -> Result<Vec<GoodsComment>> {
    let comments: Vec<GoodsComment> = sqlx::query_as(
        "SELECT id, goods_id, p.nick_name AS publisher, p.avatar AS publisher_avatar, content
        FROM mall.comments c, public.person p
        WHERE c.publisher = p.uid AND c.goods_id = $1
        ORDER BY ts DESC;",
    )
    .bind(goods_id)
    .fetch_all(db)
    .await?;
    Ok(comments)
}

pub async fn append_comment(db: &PgPool, new: &NewComment) -> Result<i32> {
    let new_comment: GoodsComment = sqlx::query_as(
        "INSERT INTO mall.comments (goods_id, publisher, content)
        VALUES ($1, $2, $3) RETURNING id;",
    )
    .bind(&new.goods_id)
    .bind(&new.publisher)
    .bind(&new.content)
    .fetch_one(db)
    .await?;
    Ok(new_comment.id)
}

pub async fn delete_comment(db: &PgPool, comment_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM mall.comments WHERE id = $1")
        .bind(comment_id)
        .execute(db)
        .await?;
    Ok(())
}
