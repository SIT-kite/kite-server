use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::mall::{MallError, NewGoods};
use crate::models::PageView;

use super::{GoodsDetail, SimpleGoods};

pub async fn get_full_goods_list(db: &PgPool, page: PageView) -> Result<Vec<SimpleGoods>> {
    let goods = sqlx::query_as(
        "SELECT id, title, cover_image, tags, price, status, publish_time
        FROM mall.goods
        ORDER BY publish_time DESC
        LIMIT $1 OFFSET $2;",
    )
    .bind(page.count(20) as i16)
    .bind(page.offset(20) as i16)
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

pub async fn get_goods_detail(db: &PgPool, goods_id: i32) -> Result<GoodsDetail> {
    let goods = sqlx::query_as(
        "SELECT
                id, title, description, status, cover_image, campus, images, tags, price,
                publisher, publish_time, wish, views, sort, features
            FROM
                mall.goods
            WHERE
                id = $1
            LIMIT 1;",
    )
    .bind(goods_id)
    .fetch_optional(db)
    .await?;
    goods.ok_or_else(|| ApiError::new(MallError::NoSuchGoods))
}

pub async fn delete_goods(db: &PgPool, goods_id: i32) -> Result<i32> {
    let result: (i32,) = sqlx::query_as("UPDATE mall.goods SET status = 0 WHERE id = $1;")
        .bind(goods_id)
        .fetch_one(db)
        .await?;
    Ok(goods_id)
}

pub async fn publish_goods(db: &PgPool, uid: i32, new: &NewGoods) -> Result<i32> {
    let returning: (i32,) = sqlx::query_as(
        "INSERT INTO mall.goods
            (title, description, status, cover_image, campus, images, tags, price,
            publisher, sort, features)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id;",
    )
    .bind(&new.title)
    .bind(&new.description)
    .bind(1i32)
    .bind(&new.cover_image)
    .bind(&new.campus)
    .bind(&new.images)
    .bind(&new.tags)
    .bind(new.price)
    .bind(uid)
    .bind(new.sort)
    .bind(&new.features)
    .fetch_one(db)
    .await?;
    Ok(returning.0)
}

pub async fn update_goods(db: &PgPool, new: &NewGoods) -> Result<i32> {
    let returning: (i32,) = sqlx::query_as(
        "UPDATE
                mall.goods
             SET
                title=$1,
                description=$2,
                status=$3,
                cover_image=$4,
                campus=$5,
                images=$6,
                tags=$7,
                price=$8,
                sort=$9,
                features=$10
             WHERE
                id=$11
             ",
    )
    .bind(&new.title)
    .bind(&new.description)
    .bind(&new.status)
    .bind(&new.cover_image)
    .bind(&new.campus)
    .bind(&new.images)
    .bind(&new.tags)
    .bind(new.price)
    .bind(new.sort)
    .bind(&new.features)
    .bind(&new.id)
    .fetch_one(db)
    .await?;
    Ok(returning.0)
}
