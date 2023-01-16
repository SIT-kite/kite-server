/// “风筝🪁时刻” 图片信息
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Picture {
    /// 图片 UUID
    #[prost(message, optional, tag = "1")]
    pub uuid: ::core::option::Option<super::template::Uuid>,
    /// 上传者 UID
    #[prost(int32, tag = "2")]
    pub uid: i32,
    /// 上传者描述， 昵称（如果有的话）或自动生成的描述
    #[prost(string, tag = "3")]
    pub publisher: ::prost::alloc::string::String,
    /// 原始图片 URL
    #[prost(string, tag = "4")]
    pub origin_url: ::prost::alloc::string::String,
    /// 缩略图 URL
    #[prost(string, tag = "5")]
    pub thumbnail: ::prost::alloc::string::String,
    /// 上传的时间戳
    #[prost(message, optional, tag = "6")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
}
/// 图片列表
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PictureListResponse {
    #[prost(message, repeated, tag = "1")]
    pub picture_list: ::prost::alloc::vec::Vec<Picture>,
}
/// TODO: 使用七牛云 SDK
/// 请求上传图片
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UploadRequest {
    /// 用户访问令牌
    #[prost(message, optional, tag = "1")]
    pub token: ::core::option::Option<super::token::UserToken>,
    /// 图片数据
    #[prost(bytes = "vec", tag = "2")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
pub mod board_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::http::Uri;
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct BoardServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl BoardServiceClient<tonic::transport::Channel> {
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

    impl<T> BoardServiceClient<T>
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> BoardServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            BoardServiceClient::new(InterceptedService::new(inner, interceptor))
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
        /// 获取公共图片列表
        pub async fn get_picture_list(
            &mut self,
            request: impl tonic::IntoRequest<super::super::template::PageOption>,
        ) -> Result<tonic::Response<super::PictureListResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/board.BoardService/GetPictureList");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取用户自己上传列表
        pub async fn get_my_upload(
            &mut self,
            request: impl tonic::IntoRequest<super::super::template::EmptyRequestWithToken>,
        ) -> Result<tonic::Response<super::PictureListResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/board.BoardService/GetMyUpload");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 上传图片
        pub async fn upload(
            &mut self,
            request: impl tonic::IntoRequest<super::UploadRequest>,
        ) -> Result<tonic::Response<super::Picture>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/board.BoardService/Upload");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}

/// Generated server implementations.
pub mod board_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with BoardServiceServer.
    #[async_trait]
    pub trait BoardService: Send + Sync + 'static {
        /// 获取公共图片列表
        async fn get_picture_list(
            &self,
            request: tonic::Request<super::super::template::PageOption>,
        ) -> Result<tonic::Response<super::PictureListResponse>, tonic::Status>;
        /// 获取用户自己上传列表
        async fn get_my_upload(
            &self,
            request: tonic::Request<super::super::template::EmptyRequestWithToken>,
        ) -> Result<tonic::Response<super::PictureListResponse>, tonic::Status>;
        /// 上传图片
        async fn upload(
            &self,
            request: tonic::Request<super::UploadRequest>,
        ) -> Result<tonic::Response<super::Picture>, tonic::Status>;
    }

    #[derive(Debug)]
    pub struct BoardServiceServer<T: BoardService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }

    struct _Inner<T>(Arc<T>);

    impl<T: BoardService> BoardServiceServer<T> {
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

    impl<T, B> tonic::codegen::Service<http::Request<B>> for BoardServiceServer<T>
    where
        T: BoardService,
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
                "/board.BoardService/GetPictureList" => {
                    #[allow(non_camel_case_types)]
                    struct GetPictureListSvc<T: BoardService>(pub Arc<T>);
                    impl<T: BoardService> tonic::server::UnaryService<super::super::template::PageOption> for GetPictureListSvc<T> {
                        type Response = super::PictureListResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::template::PageOption>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_picture_list(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPictureListSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/board.BoardService/GetMyUpload" => {
                    #[allow(non_camel_case_types)]
                    struct GetMyUploadSvc<T: BoardService>(pub Arc<T>);
                    impl<T: BoardService> tonic::server::UnaryService<super::super::template::EmptyRequestWithToken> for GetMyUploadSvc<T> {
                        type Response = super::PictureListResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::template::EmptyRequestWithToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_my_upload(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetMyUploadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/board.BoardService/Upload" => {
                    #[allow(non_camel_case_types)]
                    struct UploadSvc<T: BoardService>(pub Arc<T>);
                    impl<T: BoardService> tonic::server::UnaryService<super::UploadRequest> for UploadSvc<T> {
                        type Response = super::Picture;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::UploadRequest>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).upload(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UploadSvc(inner);
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

    impl<T: BoardService> Clone for BoardServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }

    impl<T: BoardService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: BoardService> tonic::server::NamedService for BoardServiceServer<T> {
        const NAME: &'static str = "board.BoardService";
    }
}
