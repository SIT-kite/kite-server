use sqlx::PgPool;

use crate::error::Result;
use crate::models::mall::{GoodsComment, NewComment, Comment};
use serde_json::Value;
use chrono::{DateTime, Utc, Local};
use rand::Rng;

pub async fn publish_comment(db: &PgPool, uid: i32, new: &Value) -> Result<i32> {

    //随机数
    let mut rng = rand::thread_rng();
    //获取当前时间作为编号
    let utc: DateTime<Utc> = Utc::now();
    let code  = utc.format("%Y%m%d%S").to_string();

    //编号头+年月日秒+随机三位数构成编号
    let com_code = format!("C{}{}",code,rng.gen_range(100,999));

    let parent_code;

    if new["parent_code"].is_null() {
        parent_code = "NULL"
    }else {
        parent_code = &new["parent_code"].as_str().unwrap();
    }

    let insert_publish:Option<(i32,)>  = sqlx::query_as(
        "
            INSERT INTO mall.comment(
                 com_code
                ,user_code
                ,item_code
                ,content
                ,parent_code
                ,num_like
                ,status
                ,insert_time
                ,update_time
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9);
        ",
    )
        .bind(com_code)
        .bind(uid)
        .bind(&new["item_code"].as_str().unwrap())
        .bind(&new["content"].as_str().unwrap())
        .bind(parent_code)
        .bind(0)
        .bind("Y")
        .bind(Local::now())
        .bind(Local::now())
        .fetch_optional(db)
        .await?;

    Ok(1)
}

pub async fn delete_comment(db: &PgPool, com_code: String) -> Result<()> {

    let returning: Option<(i32,)> =  sqlx::query_as(
        "
            UPDATE
                mall.comment
             SET
                status = 'N'
                ,update_time = $1
             WHERE com_code = $2
                OR parent_code = $2
             "
    )
        .bind(Local::now())
        .bind(com_code)
        .fetch_optional(db)
        .await?;

    Ok(())
}

pub async fn get_comments(db: &PgPool, item_code: String) -> Result<Vec<Comment>> {
    let goods = sqlx::query_as(
        "
            SELECT
                com_code
                ,user_code
                ,content
                ,parent_code
                ,num_like
            FROM mall.comment
            WHERE status = 'Y'
                AND item_code = $1;
            "
    )
        .bind(item_code)
        .fetch_all(db)
        .await?;

    Ok(goods)
}

pub async fn update_num_like(db: &PgPool, com_code: String) -> Result<i32> {
    //调用更新num_like的存储过程
    let update:Option<(i32,)>  = sqlx::query_as(
        "
                select update_num_like($1)
            "
    )
        .bind(com_code)
        .fetch_optional(db)
        .await?;
    Ok(1)
}