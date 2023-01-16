/// 用户访问令牌
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserToken {
    #[prost(int32, tag = "1")]
    pub uid: i32,
    #[prost(string, tag = "2")]
    pub jwt_string: ::prost::alloc::string::String,
}
