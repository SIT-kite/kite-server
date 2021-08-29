use actix_web::{get, post, web, HttpResponse, HttpRequest};

use crate::error::{ApiError, Result};
use crate::models::mall::{self, MallError, Comment_Uni, Comment, Publish, UpdateGoods, SelectGoods, PubComment};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};

use chrono::prelude::*;
use serde_json::{Value, to_string};
use std::borrow::Borrow;


pub fn is_numeric(s: &str) -> bool {
    for ch in s.chars() {
        if !ch.is_numeric() {
            return false;
        }
    }
    true
}

/// It's not a strict function for validating isbn numbers.
pub fn is_valid_isbn(isbn: &str) -> bool {
    if isbn.len() != 13 && isbn.len() != 10 {
        return false;
    }
    if !is_numeric(isbn) {
        return false;
    }
    true
}

#[get("/mall/textbook/{isbn}")]
pub async fn query_textbook(app: web::Data<AppState>, isbn: web::Path<String>) -> Result<HttpResponse> {
    let isbn = isbn.into_inner();
    if !is_valid_isbn(&isbn) {
        return Err(ApiError::new(MallError::InvalidISBN));
    }

    let textbook = mall::query_textbook_by_isbn(&app.pool, &isbn).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(textbook)))
}

#[get("/mall/sorts")]
pub async fn get_goods_sorts(app: web::Data<AppState>) -> Result<HttpResponse> {
    let sort_list = mall::get_goods_sorts(&app.pool).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(sort_list)))
}

#[derive(serde::Deserialize,Debug)]
pub struct QueryParams {
    sort: Option<i32>,
    q: Option<String>,
}

#[post("/mall/goods_list")]
pub async fn get_goods_list(
    app: web::Data<AppState>,
    form: web::Json<SelectGoods>,
) -> Result<HttpResponse> {

    let form = form.into_inner();

    let goods_list = mall::get_goods_list(&app.pool, &form).await?;

    //将取出商品转化为json
    let goods_list = serde_json::json!(goods_list);

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(goods_list)))
}

#[get("/mall/goods/{item_code}")]
pub async fn get_goods_byid(
    app: web::Data<AppState>,
    item_code: web::Path<String>
) -> Result<HttpResponse> {
    let item_code = item_code.into_inner();

    let goods_detail = mall::get_goods_detail(&app.pool,item_code).await?;

    //判断是否找到商品
    if goods_detail.is_empty() {
        return Err(ApiError::new(MallError::NoSuchGoods));
    }

    let goods_detail = serde_json::json!(goods_detail);

    Ok(HttpResponse::Ok().json(goods_detail))
}

#[post("/mall/insert_goods")]
pub async fn publish_goods(
    app: web::Data<AppState>,
/*    token: JwtToken,*/
    form: web::Json<mall::Publish>,
) -> Result<HttpResponse> {

    //拆包
    let form = form.into_inner();

    //判断是否超出长度
    if form.description.len() > 200 {
        return Err(ApiError::new(MallError::OutRange));
    }

    if form.item_name.len() > 30 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //调用敏感文字检测

    //调用数据库
    let item_code = mall::publish_goods(&app.pool, 1, &form).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(item_code)))
}

#[post("/mall/update_goods")]
pub async fn update_goods(
    app: web::Data<AppState>,
/*    token: JwtToken,*/
    form: web::Json<UpdateGoods>,
) -> Result<HttpResponse> {
    let form = form.into_inner();

    //判断是否超出长度
    if form.description.len() > 200 {
        return Err(ApiError::new(MallError::OutRange));
    }

    if form.item_name.len() > 30 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //调用敏感文字检测

    let goods_id = mall::update_goods(&app.pool, &form).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(&form.item_code)))
}

#[get("/mall/delete_goods/{pub_code}")]
pub async fn delete_goods(
    app: web::Data<AppState>,
    pub_code: web::Path<String>
) -> Result<HttpResponse> {
    let pub_code = pub_code.into_inner();

    let delete_status = mall::delete_goods(&app.pool, pub_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("delete success!")))
}

#[get("/mall/update_views/{pub_code}")]
pub async fn update_views(
    app: web::Data<AppState>,
    pub_code: web::Path<String>
) -> Result<HttpResponse> {
    let pub_code = pub_code.into_inner();

    let views_status = mall::update_views(&app.pool, pub_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("update views success!")))
}

#[post("/mall/insert_comment")]
pub async fn publish_comment(
    app: web::Data<AppState>,
/*    token: JwtToken,*/
    form: web::Json<PubComment>,
) -> Result<HttpResponse> {

    //拆包
    let form = form.into_inner();

    //判断是否超出长度
    if form.content.len() > 200 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //调用敏感文字检验


    //调用数据库
    let com_code = mall::publish_comment(&app.pool, 1, &form).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(com_code)))
}

#[get("/mall/delete_comment/{com_code}")]
pub async fn delete_comment(
    app: web::Data<AppState>,
    com_code: web::Path<String>
) -> Result<HttpResponse> {
    let com_code = com_code.into_inner();

    let delete_status = mall::delete_comment(&app.pool, com_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("delete success!")))
}

#[get("/mall/comments/{item_code}")]
pub async fn get_comments(
    app: web::Data<AppState>,
    item_code: web::Path<String>
) -> Result<HttpResponse> {
    let item_code = item_code.into_inner();

    let comments = mall::get_comments(&app.pool,item_code).await?;

    //判断是否找到商品
    if comments.is_empty() {
        return Err(ApiError::new(MallError::NoSuchGoods));
    }

    /*
        以下处理评论的父子级关系,思路为：
            优先筛选出父级评论，后根据将子级评论的parent_code进行匹配放入父级评论的child中
    */


    //存放所有父级评论Vec
    let mut comment_uni =  vec![];

    //遍历所有该商品相关评论
    for comment in comments.iter() {
        //判断parent_code是否为NULL 以判断是否为父级评论
        if comment.parent_code == "NULL" {
            let comment_parent = Comment_Uni{
                com_code: comment.com_code.clone(),
                user_code: comment.user_code.clone(),
                content: comment.content.clone(),
                parent_code: comment.parent_code.clone(),
                num_like: comment.num_like.clone(),
                children: vec![]
            };

            //放入父级评论Vec 中
            comment_uni.push(comment_parent);
        }
    }

    //再次遍历所有该商品相关评论
    for comment in comments.iter() {
        //筛选非父级评论，即parent_code 不为NULL
        if comment.parent_code != "NULL"{
            //遍历父级评论Vec
            for comment_parent in comment_uni.iter_mut(){
                //在父级评论Vec中 筛选出该子级评论的父级
                if comment_parent.com_code == comment.parent_code{
                    let comment_child = Comment{
                        com_code: comment.com_code.clone(),
                        user_code: comment.user_code.clone(),
                        content: comment.content.clone(),
                        parent_code: comment.parent_code.clone(),
                        num_like: comment.num_like.clone(),
                    };
                    //放入父级评论的child中
                    comment_parent.children.push(comment_child);
                }
            }
        }
    }

    //转化为json
    let comment_uni = serde_json::json!(comment_uni);

    Ok(HttpResponse::Ok().json(comment_uni))
}

#[get("/mall/update_num_like/{com_code}")]
pub async fn update_num_like(
    app: web::Data<AppState>,
    com_code: web::Path<String>
) -> Result<HttpResponse> {
    let com_code = com_code.into_inner();

    let views_status = mall::update_num_like(&app.pool, com_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("update num_like success!")))
}

#[post("/mall/insert_wish/{pub_code}")]
pub async fn insert_wish(
    app: web::Data<AppState>,
/*    token: JwtToken,*/
    pub_code: web::Path<String>,
) -> Result<HttpResponse> {

    //拆包
    let pub_code = pub_code.into_inner();

    //调用数据库
    let insert_status = mall::insert_wish(&app.pool, 1, pub_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("success insert!")))
}

#[get("/mall/cancel_wish/{pub_code}")]
pub async fn cancel_wish(
    app: web::Data<AppState>,
    token: JwtToken,
    pub_code: web::Path<String>
) -> Result<HttpResponse> {
    let pub_code = pub_code.into_inner();

    let delete_status = mall::cancel_wish(&app.pool, token.uid ,pub_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal("delete success!")))
}

#[get("/mall/wish/{user_code}")]
pub async fn get_wishes(
    app: web::Data<AppState>,
    user_code: web::Path<i32>
) -> Result<HttpResponse> {
    let user_code = user_code.into_inner();

    let wish_list = mall::get_wishes(&app.pool,user_code).await?;

    //判断是否找到商品
    if wish_list.is_empty() {
        return Err(ApiError::new(MallError::NoWish));
    }

    let wish_list = serde_json::json!(wish_list);

    Ok(HttpResponse::Ok().json(wish_list))
}
