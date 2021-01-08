mod comments;
mod goods;
mod textbook;

use chrono::{DateTime, Utc};
use serde::Serialize;

pub use textbook::query_textbook_by_isbn;

/// Error handled in motto module.
#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum MallError {
    #[error("教材信息库中无对应教材")]
    NoSuchTextBook = 220,
    #[error("ISBN 格式错误")]
    InvalidISBN = 221,
    #[error("找不到该商品")]
    NoSuchGoods = 222,
    #[error("商品已删除")]
    DeletedGoods = 223,
}

/* Model */
/// Each predefined textbook
#[derive(Serialize, sqlx::FromRow)]
pub struct TextBook {
    /// ISBN of the textbook
    pub isbn: Option<String>,
    /// Title
    pub title: String,
    /// Sub-title
    #[serde(rename = "subTitle")]
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
    #[serde(rename = "editionDate")]
    pub edition_date: Option<String>,
    /// Page count
    pub page: Option<i32>,
    /// The major of the book itself
    pub tag: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SimpleGoods {
    pub id: i32,
    /// Product name
    pub title: String,
    /// Cover image, used to show the whole picture
    #[serde(rename = "coverImage")]
    pub cover_image: String,
    /// Tags, like '全新', '可议价'
    pub tags: Vec<String>,
    /// Price for selling
    pub price: f32,
    /// Status
    pub status: i16,
}

#[derive(Serialize, sqlx::FromRow)]
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
    #[serde(rename = "coverImage")]
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
    #[serde(rename = "publishTime")]
    pub publish_time: DateTime<Utc>,
    /// The count of person who want to buy and have gotten the contact of seller.
    pub wish: i16,
    /// Total views
    pub views: i32,
    /// Sort id
    pub sort: i32,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct GoodsComment {
    pub id: i32,
    #[serde(rename = "goodsId")]
    pub goods_id: i32,
    /// Publisher's nick name
    #[serde(rename = "publisherName")]
    pub publisher: String,
    /// A url to publisher avatar
    pub publisher_avatar: String,
    /// Comment content
    pub content: String,
}
