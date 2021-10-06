use crate::bridge::{
    BookHoldingRequest, HostError, RequestFrame, RequestPayload, ResponsePayload, SearchLibraryRequest,
    SearchWay, SortOrder, SortWay,
};
use crate::error::{ApiError, Result};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryBookQuery {
    /// 搜索关键字
    keyword: String,
    /// 搜索方式
    search_way: Option<String>,
    /// 搜索结果的排序方式
    sort_way: Option<String>,
    /// 搜索结果的升降序方式
    sort_order: Option<String>,
}

fn to_search_library_request(query: LibraryBookQuery, page: PageView) -> SearchLibraryRequest {
    SearchLibraryRequest {
        keyword: query.keyword,
        rows: page.count(20),
        page: page.index() as u32,
        search_way: query
            .search_way
            .and_then(|x| SearchWay::from_str(&x).ok())
            .unwrap_or(SearchWay::Any),
        sort_way: query
            .sort_way
            .and_then(|x| SortWay::from_str(&x).ok())
            .unwrap_or(SortWay::MatchScore),
        sort_order: query
            .sort_order
            .and_then(|x| SortOrder::from_str(&x).ok())
            .unwrap_or(SortOrder::Desc),
    }
}

#[get("/library/book")]
pub async fn query_books(
    app: web::Data<AppState>,
    page: web::Query<PageView>,
    query: web::Query<LibraryBookQuery>,
) -> Result<HttpResponse> {
    let request = to_search_library_request(query.into_inner(), page.into_inner());

    let agents = &app.agents;
    let payload = RequestPayload::SearchLibrary(request);
    let request = RequestFrame::new(payload);

    let response = agents.request(request).await??;
    if let ResponsePayload::SearchLibrary(result) = response {
        Ok(HttpResponse::Ok().json(ApiResponse::normal(result)))
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

#[get("/library/book/{book_id}/holding")]
pub async fn query_book_holding(
    app: web::Data<AppState>,
    book_id: web::Path<String>,
) -> Result<HttpResponse> {
    let book_id = book_id.into_inner();
    if book_id.len() != book_id.chars().filter(char::is_ascii_digit).count() {
        return Err(ApiError::new(CommonError::Parameter));
    }
    let request = BookHoldingRequest {
        book_id_list: vec![book_id.clone()],
    };
    let agents = &app.agents;
    let payload = RequestPayload::BookHoldingInfo(request);
    let request = RequestFrame::new(payload);

    let response = agents.request(request).await??;

    if let ResponsePayload::BookHoldingInfo(mut result) = response {
        let result = result
            .holding_previews
            .remove(&book_id)
            .unwrap_or_else(std::vec::Vec::new);
        let response = json!({
            "holdingList": result,
        });
        Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
    } else {
        Err(ApiError::new(HostError::Mismatched))
    }
}

#[get("/library/book/{book_id}")]
pub async fn query_book_detail(
    _app: web::Data<AppState>,
    _book_id: web::Path<String>,
) -> Result<HttpResponse> {
    // let request = BookHoldingRequest {
    //     book_id_list: vec![book_id.into_inner()],
    // };
    // let agents = &app.agents;
    // let payload = RequestPayload::BookHoldingInfo(request);
    // let request = RequestFrame::new(payload);
    //
    // let response = agents.request(request).await??;
    //
    // if let ResponsePayload::BookHoldingInfo(result) = response {
    //     Ok(HttpResponse::Ok().json(ApiResponse::normal(result)))
    // } else {
    //     Err(ApiError::new(HostError::Mismatched))
    // }
    Err(ApiError::new(CommonError::Forbidden))
}
