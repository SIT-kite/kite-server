#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClassroomQuery {
    /// 教学楼名称，如 "一教"
    #[prost(string, optional, tag = "1")]
    pub building: ::core::option::Option<::prost::alloc::string::String>,
    /// 教学区域名称，如 "A", "B"
    #[prost(string, optional, tag = "2")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    /// 校区
    #[prost(enumeration = "super::typing::Campus", optional, tag = "3")]
    pub campus: ::core::option::Option<i32>,
    /// 当前学期的周序号，一般为 1-18
    #[prost(int32, tag = "4")]
    pub week: i32,
    /// 星期几，取值 1 - 7
    #[prost(int32, tag = "5")]
    pub day: i32,
    /// 期望有空闲的时间，使用二进制位表示。
    /// 如果某一位（从右，从 0 开始计数）为 1，比如从右数第 1 位为 1, 那么表示希望第一节课空闲
    ///
    /// 值 110b 表示希望 1-2 节课，即请求 8:20-9:55 空闲的教室，如果当前值省略，默认不筛选时间。
    #[prost(int32, optional, tag = "6")]
    pub time_flag: ::core::option::Option<i32>,
}
/// 教室信息
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Classroom {
    /// 教室名称，如 "C103"
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// 教室使用情况，同 time_flag, 是一个用位表示的标记
    #[prost(int32, tag = "2")]
    pub busy_flag: i32,
    /// 教室容量，部分教室暂缺
    #[prost(int32, optional, tag = "3")]
    pub capacity: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClassroomListResponse {
    #[prost(message, repeated, tag = "1")]
    pub classroom_list: ::prost::alloc::vec::Vec<Classroom>,
}
/// Generated server implementations.
pub mod classroom_browser_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with ClassroomBrowserServiceServer.
    #[async_trait]
    pub trait ClassroomBrowserService: Send + Sync + 'static {
        /// 根据给定位置和时间，获取空教室列表
        async fn get_available_classroom(
            &self,
            request: tonic::Request<super::ClassroomQuery>,
        ) -> Result<tonic::Response<super::ClassroomListResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ClassroomBrowserServiceServer<T: ClassroomBrowserService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ClassroomBrowserService> ClassroomBrowserServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ClassroomBrowserServiceServer<T>
    where
        T: ClassroomBrowserService,
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
                "/classroom_browser.ClassroomBrowserService/GetAvailableClassroom" => {
                    #[allow(non_camel_case_types)]
                    struct GetAvailableClassroomSvc<T: ClassroomBrowserService>(pub Arc<T>);
                    impl<T: ClassroomBrowserService> tonic::server::UnaryService<super::ClassroomQuery> for GetAvailableClassroomSvc<T> {
                        type Response = super::ClassroomListResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::ClassroomQuery>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_available_classroom(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAvailableClassroomSvc(inner);
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
    impl<T: ClassroomBrowserService> Clone for ClassroomBrowserServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ClassroomBrowserService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ClassroomBrowserService> tonic::server::NamedService for ClassroomBrowserServiceServer<T> {
        const NAME: &'static str = "classroom_browser.ClassroomBrowserService";
    }
}
