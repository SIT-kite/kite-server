use actix_web::{delete, get, post, put, web, HttpResponse};

use crate::error::{ApiError, Result};
use crate::models::mall::{
    self, Comment, CommentUni, MallError, PubComment, PubWish, SelectGoods, UpdateGoods,
};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};
use crate::models::user;
use wechat_sdk::wechat::Check;

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

#[get("/mall/sort")]
pub async fn get_goods_sorts(app: web::Data<AppState>) -> Result<HttpResponse> {
    let sort_list = mall::get_goods_sorts(&app.pool).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(sort_list)))
}

#[derive(serde::Deserialize, Debug)]
pub struct QueryParams {
    sort: Option<i32>,
    q: Option<String>,
}

#[get("/mall/goods")]
pub async fn get_goods_list(
    app: web::Data<AppState>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let form = SelectGoods {
        sort: None,
        keyword: "".to_string(),
    };
    let goods_list = mall::get_goods_list(&app.pool, &form, page.into_inner()).await?;

    let response = serde_json::json!({
        "goods": goods_list,
    });

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[get("/mall/goods/sort/{sort}")]
pub async fn get_goods_list_by_sort(
    app: web::Data<AppState>,
    sort: web::Path<i32>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let sort = sort.into_inner();

    let form = SelectGoods {
        sort: sort.checked_abs(),
        keyword: "".to_string(),
    };

    let goods_list = mall::get_goods_list(&app.pool, &form, page.into_inner()).await?;

    let response = serde_json::json!({
        "goods": goods_list,
    });

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[get("/mall/goods/like/{keyword}")]
pub async fn get_goods_list_by_keyword(
    app: web::Data<AppState>,
    keyword: web::Path<String>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let keyword = keyword.into_inner();

    let form = SelectGoods { sort: None, keyword };

    let goods_list = mall::get_goods_list(&app.pool, &form, page.into_inner()).await?;

    let response = serde_json::json!({
        "goods": goods_list,
    });

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[get("/mall/goods/{item_code}")]
pub async fn get_goods_by_id(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    item_code: web::Path<String>,
) -> Result<HttpResponse> {
    let item_code = item_code.into_inner();
    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    //获取商品详情
    let detail = mall::get_goods_detail(&app.pool, &item_code).await?;

    //插入观看日志
    mall::insert_view_log(&app.pool, uid, &item_code).await?;

    let response = serde_json::json!({
        "detail": detail,
    });

    Ok(HttpResponse::Ok().json(response))
}

#[post("/mall/goods")]
pub async fn publish_goods(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Json<mall::Publish>,
) -> Result<HttpResponse> {
    let mut form = form.into_inner();
    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    // 判断是否超出长度
    if form.description.len() > 200 || form.item_name.len() > 30 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //获取openid
    let openid = user::get_open_id(&app.pool,uid).await?;
    //拼接验证内容(item_name + description)
    let content = format!("{}{}",form.item_name,form.description);

    //内容违规检测
    let check_response = app.wx_client.msg_sec_check(openid,"1".to_string() ,content).await?;

    //向form中添加内容验证结果
    form.suggest = Some(check_response.result.suggest.clone());
    form.label = Some(check_response.result.label.clone());

    // 请求添加新商品
    let item_code = mall::publish_goods(&app.pool, uid, &form).await?;
    let response = serde_json::json!({
        "code": item_code,
    });

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[put("/mall/goods")]
pub async fn update_goods(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Json<UpdateGoods>,
) -> Result<HttpResponse> {
    let mut form = form.into_inner();

    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    // 判断是否超出长度
    if form.description.len() > 200 || form.item_name.len() > 30 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //获取openid
    let openid = user::get_open_id(&app.pool,uid).await?;
    //拼接验证内容(item_name + description)
    let content = format!("{}{}",form.item_name,form.description);

    //内容违规检测
    let check_response = app.wx_client.msg_sec_check(openid,"1".to_string() ,content).await?;

    //向form中添加内容验证结果
    form.suggest = Some(check_response.result.suggest.clone());
    form.label = Some(check_response.result.label.clone());

    // 权限校验
    let pub_code = mall::check_goods(&app.pool, uid, &form).await?;

    form.pub_code = Some(pub_code);

    //修改发布以及商品信息
    let pub_code = mall::update_publish(&app.pool, &form).await?;
    let goods_id = mall::update_goods(&app.pool, &form).await?;

    let response = serde_json::json!({
        "goods_id": goods_id,
        "pub_code": pub_code
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[delete("/mall/goods/{pub_code}")]
pub async fn delete_goods(
    app: web::Data<AppState>,
    pub_code: web::Path<String>,
) -> Result<HttpResponse> {
    let pub_code = pub_code.into_inner();

    let _ = mall::delete_goods(&app.pool, pub_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[post("/mall/comment")]
pub async fn publish_comment(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Json<PubComment>,
) -> Result<HttpResponse> {
    let mut form = form.into_inner();
    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    // 判断是否超出长度
    if form.content.len() > 200 {
        return Err(ApiError::new(MallError::OutRange));
    }

    //获取openid
    let openid = user::get_open_id(&app.pool,uid).await?;

    //内容违规检测
    let check_response = app.wx_client.msg_sec_check(openid,"1".to_string() ,form.content.clone()).await?;

    //向form中添加内容验证结果
    form.suggest = Some(check_response.result.suggest.clone());
    form.label = Some(check_response.result.label.clone());

    // 数据库插入评论
    let com_code = mall::publish_comment(&app.pool, uid, &form).await?;
    let response = serde_json::json!({
        "code": com_code,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[delete("/mall/comment/{com_code}")]
pub async fn delete_comment(
    app: web::Data<AppState>,
    com_code: web::Path<String>,
) -> Result<HttpResponse> {
    let _ = mall::delete_comment(&app.pool, com_code.into_inner()).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[get("/mall/comment/{item_code}")]
pub async fn get_comments(
    app: web::Data<AppState>,
    item_code: web::Path<String>,
) -> Result<HttpResponse> {
    let item_code = item_code.into_inner();
    let comments = mall::get_comments(&app.pool, item_code).await?;

    // 判断是否找到商品
    if comments.is_empty() {
        return Err(ApiError::new(MallError::NoSuchGoods));
    }

    /*
    以下处理评论的父子级关系, 思路为:
    优先筛选出父级评论，后根据将子级评论的parent_code进行匹配放入父级评论的child中
    */

    // 存放所有父级评论Vec
    let mut comment_uni = vec![];

    // 遍历所有该商品相关评论
    for comment in comments.iter() {
        // 判断parent_code是否为NULL 以判断是否为父级评论
        if comment.parent_code == "NULL" {
            let comment_parent = CommentUni {
                com_code: comment.com_code.clone(),
                user_code: comment.user_code,
                content: comment.content.clone(),
                parent_code: comment.parent_code.clone(),
                num_like: comment.num_like,
                children: vec![],
            };
            // 放入父级评论Vec 中
            comment_uni.push(comment_parent);
        }
    }

    // 再次遍历所有该商品相关评论
    for comment in comments.iter() {
        // 筛选非父级评论，即parent_code 不为NULL
        if comment.parent_code != "NULL" {
            // 遍历父级评论Vec
            for comment_parent in comment_uni.iter_mut() {
                // 在父级评论Vec中 筛选出该子级评论的父级
                if comment_parent.com_code == comment.parent_code {
                    let comment_child = Comment {
                        com_code: comment.com_code.clone(),
                        user_code: comment.user_code,
                        content: comment.content.clone(),
                        parent_code: comment.parent_code.clone(),
                        num_like: comment.num_like,
                    };
                    // 放入父级评论的child中
                    comment_parent.children.push(comment_child);
                }
            }
        }
    }

    // 转化为json
    let comment_uni = serde_json::json!(comment_uni);

    Ok(HttpResponse::Ok().json(comment_uni))
}

#[put("/mall/comment/like/{com_code}")]
pub async fn update_num_like(
    app: web::Data<AppState>,
    com_code: web::Path<String>,
) -> Result<HttpResponse> {
    let com_code = com_code.into_inner();

    let _ = mall::update_num_like(&app.pool, com_code).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[post("/mall/wish")]
pub async fn append_wish(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    form: web::Json<PubWish>,
) -> Result<HttpResponse> {
    let pub_code = form.into_inner().pub_code;
    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    //校验商品信息是否存在

    let item_code = mall::check_publish(&app.pool, &pub_code).await?;

    // 调用数据库
    let _ = mall::insert_wish(&app.pool, uid, &pub_code).await?;

    Ok(HttpResponse::Ok().json(item_code))
}

#[delete("/mall/wish/{pub_code}")]
pub async fn cancel_wish(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    pub_code: web::Path<String>,
) -> Result<HttpResponse> {
    let pub_code = pub_code.into_inner();
    let uid = token
        .map(|token| token.uid)
        .ok_or_else(|| ApiError::new(CommonError::LoginNeeded))?;

    let _ = mall::cancel_wish(&app.pool, uid, pub_code).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[get("/mall/wish/{user_code}")]
pub async fn get_wishes(app: web::Data<AppState>, user_code: web::Path<i32>) -> Result<HttpResponse> {
    let user_code = user_code.into_inner();
    let wish_list = mall::get_user_wishes(&app.pool, user_code).await?;

    let response = serde_json::json!({
        "wishList": wish_list,
    });
    Ok(HttpResponse::Ok().json(response))
}
