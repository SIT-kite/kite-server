/// 用户 “扫福” 记录
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanRecord {
    /// 用户 ID
    /// int32 uid = 1;
    /// “扫福” 结果类型
    #[prost(enumeration = "ScanResult", tag = "2")]
    pub r#type: i32,
    /// 抽到的卡类型。暂且考虑到扩展性，使用 int 类型表示
    #[prost(int32, optional, tag = "3")]
    pub card: ::core::option::Option<i32>,
    /// 触发的时间
    #[prost(message, optional, tag = "4")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
}
/// 用户卡片
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Card {
    /// 卡片类型
    #[prost(int32, tag = "1")]
    pub card_type: i32,
    /// 抽卡时间
    #[prost(message, optional, tag = "2")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardListResponse {
    #[prost(message, repeated, tag = "1")]
    pub card_list: ::prost::alloc::vec::Vec<Card>,
}
/// 用户 “扫福” 结果
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ScanResult {
    /// 没有识别到校徽
    NoBadge = 0,
    /// 当日领福卡次数已达到限制
    ReachLimit = 1,
    /// 没有抽中
    NoCard = 2,
    /// 抽中了
    WinCard = 3,
}
impl ScanResult {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ScanResult::NoBadge => "NoBadge",
            ScanResult::ReachLimit => "ReachLimit",
            ScanResult::NoCard => "NoCard",
            ScanResult::WinCard => "WinCard",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NoBadge" => Some(Self::NoBadge),
            "ReachLimit" => Some(Self::ReachLimit),
            "NoCard" => Some(Self::NoCard),
            "WinCard" => Some(Self::WinCard),
            _ => None,
        }
    }
}
/// Generated server implementations.
pub mod badge_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with BadgeServiceServer.
    #[async_trait]
    pub trait BadgeService: Send + Sync + 'static {
        /// 获取用户所抽到的所有卡片
        async fn get_user_card_storage(
            &self,
            request: tonic::Request<super::super::template::EmptyRequest>,
        ) -> Result<tonic::Response<super::CardListResponse>, tonic::Status>;
        /// 记录用户分享事件
        /// 该方法用于增加用户抽卡次数（2022春节）
        async fn append_share_log(
            &self,
            request: tonic::Request<super::super::template::EmptyRequest>,
        ) -> Result<tonic::Response<super::super::template::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct BadgeServiceServer<T: BadgeService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BadgeService> BadgeServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BadgeServiceServer<T>
    where
        T: BadgeService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/badge.BadgeService/GetUserCardStorage" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserCardStorageSvc<T: BadgeService>(pub Arc<T>);
                    impl<T: BadgeService> tonic::server::UnaryService<super::super::template::EmptyRequest> for GetUserCardStorageSvc<T> {
                        type Response = super::CardListResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::template::EmptyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_user_card_storage(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserCardStorageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/badge.BadgeService/AppendShareLog" => {
                    #[allow(non_camel_case_types)]
                    struct AppendShareLogSvc<T: BadgeService>(pub Arc<T>);
                    impl<T: BadgeService> tonic::server::UnaryService<super::super::template::EmptyRequest> for AppendShareLogSvc<T> {
                        type Response = super::super::template::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::template::EmptyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).append_share_log(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppendShareLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: BadgeService> Clone for BadgeServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: BadgeService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BadgeService> tonic::server::NamedService for BadgeServiceServer<T> {
        const NAME: &'static str = "badge.BadgeService";
    }
}
