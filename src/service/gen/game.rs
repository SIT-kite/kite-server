/// 排名中的单项
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RankingItem {
    /// 名次
    #[prost(int32, tag = "1")]
    pub ranking: i32,
    /// 用户描述。昵称或用户自定义描述
    #[prost(string, tag = "2")]
    pub user_description: ::prost::alloc::string::String,
    /// 游戏类型
    #[prost(enumeration = "GameType", tag = "3")]
    pub r#type: i32,
    /// 得分
    #[prost(int32, tag = "4")]
    pub score: i32,
}
/// 单条游戏记录
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameRecord {
    /// 产生记录的时间戳
    #[prost(message, optional, tag = "1")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
    /// 游戏类型
    #[prost(enumeration = "GameType", tag = "2")]
    pub r#type: i32,
    /// 得分值
    #[prost(int32, tag = "3")]
    pub score: i32,
    /// 游戏用时
    #[prost(int32, optional, tag = "4")]
    pub time_cost: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecordListRequest {
    /// 用户凭据
    #[prost(message, optional, tag = "1")]
    pub token: ::core::option::Option<super::token::UserToken>,
    /// 请求分页信息
    #[prost(message, optional, tag = "2")]
    pub page: ::core::option::Option<super::template::PageOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecordListResponse {
    #[prost(message, repeated, tag = "1")]
    pub game_record_list: ::prost::alloc::vec::Vec<GameRecord>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicRankingRequest {
    #[prost(enumeration = "GameType", tag = "1")]
    pub r#type: i32,
    #[prost(message, optional, tag = "2")]
    pub page: ::core::option::Option<super::template::PageOption>,
}
/// 游戏类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GameType {
    G2048 = 0,
    Wordle = 1,
    ComposeSit = 2,
    Tetris = 3,
}
impl GameType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GameType::G2048 => "g2048",
            GameType::Wordle => "wordle",
            GameType::ComposeSit => "compose_sit",
            GameType::Tetris => "tetris",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "g2048" => Some(Self::G2048),
            "wordle" => Some(Self::Wordle),
            "compose_sit" => Some(Self::ComposeSit),
            "tetris" => Some(Self::Tetris),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod game_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct GameServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GameServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GameServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GameServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            GameServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// 保存用户游戏记录
        pub async fn save_score(
            &mut self,
            request: impl tonic::IntoRequest<super::GameRecord>,
        ) -> Result<tonic::Response<super::super::template::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/game.GameService/SaveScore",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取公共游戏排名列表
        pub async fn get_public_ranking(
            &mut self,
            request: impl tonic::IntoRequest<super::PublicRankingRequest>,
        ) -> Result<tonic::Response<super::RecordListResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/game.GameService/GetPublicRanking",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取个人游戏记录
        pub async fn get_my_record_list(
            &mut self,
            request: impl tonic::IntoRequest<super::RecordListRequest>,
        ) -> Result<tonic::Response<super::RecordListResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/game.GameService/GetMyRecordList",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod game_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with GameServiceServer.
    #[async_trait]
    pub trait GameService: Send + Sync + 'static {
        /// 保存用户游戏记录
        async fn save_score(
            &self,
            request: tonic::Request<super::GameRecord>,
        ) -> Result<tonic::Response<super::super::template::Empty>, tonic::Status>;
        /// 获取公共游戏排名列表
        async fn get_public_ranking(
            &self,
            request: tonic::Request<super::PublicRankingRequest>,
        ) -> Result<tonic::Response<super::RecordListResponse>, tonic::Status>;
        /// 获取个人游戏记录
        async fn get_my_record_list(
            &self,
            request: tonic::Request<super::RecordListRequest>,
        ) -> Result<tonic::Response<super::RecordListResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct GameServiceServer<T: GameService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: GameService> GameServiceServer<T> {
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
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GameServiceServer<T>
    where
        T: GameService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/game.GameService/SaveScore" => {
                    #[allow(non_camel_case_types)]
                    struct SaveScoreSvc<T: GameService>(pub Arc<T>);
                    impl<T: GameService> tonic::server::UnaryService<super::GameRecord>
                    for SaveScoreSvc<T> {
                        type Response = super::super::template::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GameRecord>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).save_score(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SaveScoreSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/game.GameService/GetPublicRanking" => {
                    #[allow(non_camel_case_types)]
                    struct GetPublicRankingSvc<T: GameService>(pub Arc<T>);
                    impl<
                        T: GameService,
                    > tonic::server::UnaryService<super::PublicRankingRequest>
                    for GetPublicRankingSvc<T> {
                        type Response = super::RecordListResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PublicRankingRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_public_ranking(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPublicRankingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/game.GameService/GetMyRecordList" => {
                    #[allow(non_camel_case_types)]
                    struct GetMyRecordListSvc<T: GameService>(pub Arc<T>);
                    impl<
                        T: GameService,
                    > tonic::server::UnaryService<super::RecordListRequest>
                    for GetMyRecordListSvc<T> {
                        type Response = super::RecordListResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RecordListRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_my_record_list(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetMyRecordListSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: GameService> Clone for GameServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: GameService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: GameService> tonic::server::NamedService for GameServiceServer<T> {
        const NAME: &'static str = "game.GameService";
    }
}
