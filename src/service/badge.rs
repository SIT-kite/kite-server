use crate::S;

use volo_gen::kite::badge::*;
use volo_gen::kite::template;

#[volo::async_trait]
impl BadgeService for S {

    async fn get_user_card_storage(
        &self,
        _req: volo_grpc::Request<template::EmptyRequestWithToken>,
    ) -> Result<volo_grpc::Response<CardListResponse>, volo_grpc::Status> {
        let request = _req.get_ref();

        Ok(volo_grpc::Response::new(CardListResponse {
            card_list: vec![]
        }))
    }

    async fn append_share_log(
        &self,
        _req: volo_grpc::Request<template::EmptyRequestWithToken>,
    ) -> Result<volo_grpc::Response<template::Empty>, volo_grpc::Status> {
        let request = _req.get_ref();

        Ok(volo_grpc::Response::new(template::Empty {}))
    }
}
