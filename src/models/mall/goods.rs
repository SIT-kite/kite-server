use super::{GoodsDetail, SimpleGoods};
use crate::error::{ApiError, Result};
use crate::models::mall::MallError;
use crate::models::PageView;
use sqlx::PgPool;

pub async fn get_full_goods_list(db: &PgPool, page: PageView) -> Result<Vec<SimpleGoods>> {
    let goods = sqlx::query_as!(
        SimpleGoods,
        "SELECT id, title, cover_image, tags, price, status
        FROM mall.goods
        ORDER BY publish_time DESC
        LIMIT $1 OFFSET $2;",
        page.count(10) as i16,
        page.offset(10) as i64
    )
    .fetch_all(db)
    .await?;
    Ok(goods)
}

pub async fn get_goods_list(db: &PgPool, sort: i32, page: PageView) -> Result<Vec<SimpleGoods>> {
    let goods = sqlx::query_as!(
        SimpleGoods,
        "SELECT id, title, cover_image, tags, price, status
        FROM mall.goods
        WHERE sort = $1
        ORDER BY publish_time DESC
        LIMIT $2 OFFSET $3;",
        sort,
        page.count(10) as i16,
        page.offset(10) as i64
    )
    .fetch_all(db)
    .await?;
    Ok(goods)
}

pub async fn query_goods(db: &PgPool, keyword: &str) -> Result<Vec<SimpleGoods>> {
    let query_string = format!("%{}%", keyword);
    let goods = sqlx::query_as!(
        SimpleGoods,
        "SELECT id, title, cover_image, tags, price, status
        FROM mall.goods
        WHERE title LIKE $1 AND status != 0
        ORDER BY publish_time DESC;",
        query_string
    )
    .fetch_all(db)
    .await?;
    Ok(goods)
}

pub async fn get_goods_detail(db: &PgPool, goods_id: i32) -> Result<GoodsDetail> {
    let goods = sqlx::query_as!(
        GoodsDetail,
        "SELECT
                id, title, description, status, cover_image, campus, images, tags, price,
                publisher, publish_time, wish, views, sort, features
            FROM
                mall.goods
            WHERE
                id = $1
            LIMIT 1;",
        goods_id
    )
    .fetch_optional(db)
    .await?;
    goods.ok_or(ApiError::new(MallError::NoSuchGoods))
}

pub async fn delete_goods(db: &PgPool, goods_id: i32) -> Result<()> {
    let result: Option<_> = sqlx::query!("UPDATE mall.goods SET status = 0 WHERE id = $1;", goods_id)
        .fetch_optional(db)
        .await?;
    result.map(|_| ()).ok_or(ApiError::new(MallError::NoSuchGoods))
}

pub async fn publish_goods(db: &PgPool, new: &GoodsDetail) -> Result<i32> {
    let returning = sqlx::query!(
        "INSERT INTO mall.goods
            (title, description, status, cover_image, campus, images, tags, price,
            publisher, sort, features)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id;",
        new.title,
        new.description,
        new.status,
        new.cover_image,
        new.campus,
        &new.images,
        &new.tags,
        new.price,
        new.publisher,
        new.sort,
        &new.features
    )
    .fetch_one(db)
    .await?;
    Ok(returning.id)
}
