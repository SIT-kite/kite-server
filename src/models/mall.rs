use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub use comments::*;
pub use favorite::*;
pub use goods::*;
pub use sort::*;
pub use textbook::*;
pub use views::*;
pub use wish::*;

mod comments;
mod favorite;
mod goods;
mod sort;
mod textbook;
mod views;
mod wish;

/// Error handled in motto module.
#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum MallError {
    #[error("教材信息库中无对应教材")]
    NoSuchTextBook = 220,
    #[error("ISBN 格式错误")]
    InvalidISBN = 221,
    #[error("找不到该商品")]
    NoSuchGoods = 222,
    #[error("缺少必要参数")]
    MissingParam = 223,
    #[error("长度超出规定范围")]
    OutRange = 224,
    #[error("用户输入错误或该用户无收藏")]
    NoWish = 225,
    #[error("该用户无相应商品")]
    NoUserGood = 226,
}

/* Model */
/// Each predefined textbook
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TextBook {
    /// ISBN of the textbook
    pub isbn: Option<String>,
    /// Title
    pub title: String,
    /// Sub-title
    pub sub_title: Option<String>,
    /// Publisher's full name
    pub press: String,
    /// Author
    pub author: Option<String>,
    /// Translator (if it is a translated book)
    pub translator: Option<String>,
    /// Official price
    pub price: Option<f32>,
    /// Edition
    pub edition: Option<String>,
    /// Publication year and month
    pub edition_date: Option<String>,
    /// Page count
    pub page: Option<i32>,
    /// The major of the book itself
    pub tag: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Sorts {
    /// Sort id
    pub id: i32,
    /// Sort name
    pub title: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SimpleGoods {
    pub id: i32,
    /// Product name
    pub title: String,
    /// Cover image, used to show the whole picture
    pub cover_image: String,
    /// Tags, like '全新', '可议价'
    pub tags: Vec<String>,
    /// Price for selling
    pub price: f32,
    /// Status
    pub status: i16,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GoodsDetail {
    pub id: i32,
    /// Product name
    pub title: String,
    /// Product description and transaction requirements
    pub description: Option<String>,
    /// Goods status:
    /// Normal, Sold or disabled.
    pub status: i16,
    /// Cover image, used to show the whole picture
    pub cover_image: String,
    /// Campus name.
    pub campus: String,
    /// Product detailed picture introduction
    pub images: Vec<String>,
    /// Tags, like '全新', '可议价'
    pub tags: Vec<String>,
    /// Features
    pub features: serde_json::Value,
    /// Price for selling
    pub price: f32,
    /// Uid of the Publisher
    pub publisher: i32,
    /// Submit and publish time
    pub publish_time: DateTime<Local>,
    /// The count of person who want to buy and have gotten the contact of seller.
    pub wish: i16,
    /// Total views
    pub views: i32,
    /// Sort id
    pub sort: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewGoods {
    pub id: i32,
    /// Product name
    pub title: String,
    /// Product description and transaction requirements
    pub description: Option<String>,
    /// Cover image, used to show the whole picture
    pub cover_image: String,
    /// Campus name.
    pub campus: String,
    /// Product detailed picture introduction
    pub images: Vec<String>,
    /// Tags, like '全新', '可议价'
    pub tags: Vec<String>,
    /// Price for selling
    pub price: f32,
    /// Sort id
    pub sort: i32,
    /// Features
    pub features: serde_json::Value,
    /// Is Hidden
    pub status: i32,
}

/* Comments */
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GoodsComment {
    pub id: i32,
    pub goods_id: i32,
    /// Publisher's nick name
    pub publisher: String,
    /// A url to publisher avatar
    pub publisher_avatar: String,
    /// Comment content
    pub content: String,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NewComment {
    pub id: i32,
    pub goods_id: i32,
    /// Publisher's uid
    pub publisher: i32,
    /// Comment content
    pub content: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Favorites {
    /// Goods id
    pub goods: i32,
    /// Goods title
    pub title: String,
    /// Goods image
    pub image: String,
    /// Favorite timestamp
    pub ts: DateTime<Local>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Views {
    /// Person uid
    pub person: i32,
    /// Goods id
    pub goods: i32,
    /// Favorite timestamp
    pub ts: DateTime<Local>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Wishes {
    /// Person uid
    pub person: i32,
    /// Goods id
    pub goods: i32,
    /// Favorite timestamp
    pub ts: DateTime<Local>,
}

//首页商品信息
#[derive(Serialize, sqlx::FromRow)]
pub struct CoverInfo {
    pub pub_code: String,
    pub item_code: String,
    pub views: i64,
    pub item_name: String,
    pub price: f64,
    pub cover_image: String,
}

//商品详情信息
#[derive(Serialize, sqlx::FromRow)]
pub struct DetailInfo {
    pub item_name: String,
    pub description: String,
    pub price: f64,
    pub images: String,
}

//评论列表
#[derive(Serialize, sqlx::FromRow)]
pub struct Comment {
    pub com_code: String,
    pub user_code: i32,
    pub content: String,
    pub parent_code: String,
    pub num_like: i32,
}

//用于父级评论整合子级评论
#[derive(Serialize, sqlx::FromRow)]
pub struct CommentUni {
    pub com_code: String,
    pub user_code: i32,
    pub content: String,
    pub parent_code: String,
    pub num_like: i32,
    pub children: Vec<Comment>,
}

//收藏列表
#[derive(Serialize, sqlx::FromRow)]
pub struct Wish {
    pub pub_code: String,
    pub item_code: String,
    pub views: i64,
    pub status: String,
    pub item_name: String,
    pub price: f64,
    pub cover_image: String,
}

//发布商品参数
#[derive(Deserialize, sqlx::FromRow)]
pub struct Publish {
    pub item_name: String,
    pub description: String,
    pub price: f32,
    pub images: String,
    pub cover_image: String,
    pub campus: String,
    pub sort: i32,
}

//更新商品参数
#[derive(Deserialize, sqlx::FromRow)]
pub struct UpdateGoods {
    pub item_code: String,
    pub item_name: String,
    pub description: String,
    pub price: f32,
    pub images: String,
    pub cover_image: String,
    pub sort: i32,
}

//查询商品列表参数
#[derive(Deserialize, sqlx::FromRow)]
pub struct SelectGoods {
    pub sort: Option<i32>,
    pub keyword: String,
}

//商品发布参数
#[derive(Deserialize, sqlx::FromRow)]
pub struct PubComment {
    pub item_code: String,
    pub content: String,
    pub parent_code: Option<String>,
}

//收藏表发布参数
#[derive(Deserialize, sqlx::FromRow)]
pub struct PubWish {
    pub pub_code: String,
}
