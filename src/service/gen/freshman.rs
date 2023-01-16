/// 新生数据表中的基本个人信息
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MySelf {
    /// 姓名
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// 在小程序时期，由于每位用户对应一个 openid, 进而注册产生 uid
    /// 该字段用于统计注册的新生数。现已废弃。
    /// string uid = 2;
    /// 9或10位学号
    #[prost(string, tag = "3")]
    pub student_id: ::prost::alloc::string::String,
    /// 验证码（密码），由身份证号生成
    /// string secret = 4;
    /// 学院名称
    #[prost(string, tag = "5")]
    pub college: ::prost::alloc::string::String,
    /// 专业名称（注意，可能较长）
    #[prost(string, tag = "6")]
    pub major: ::prost::alloc::string::String,
    /// 校区. TODO: 该字段为后端返回得到，暂时使用 string 表示
    #[prost(string, tag = "7")]
    pub campus: ::prost::alloc::string::String,
    /// 宿舍楼号。部分徐汇校区的寝室楼形如 “南-18”，需要使用字符串表示
    #[prost(string, tag = "8")]
    pub building: ::prost::alloc::string::String,
    /// 寝室号，如 201, 302...
    #[prost(int32, tag = "9")]
    pub room: i32,
    /// 床号，一般是 1-5
    #[prost(int32, tag = "10")]
    pub bed_index: i32,
    /// 是否对 “可能认识的人” 可见
    #[prost(bool, tag = "13")]
    pub visible: bool,
    /// 联系方式， JSON 字符串，注意校验其结构
    #[prost(string, optional, tag = "14")]
    pub contact: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `MySelf`.
pub mod my_self {
    /// 辅导员信息
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Counselor {
        /// 姓名
        #[prost(string, tag = "11")]
        pub name: ::prost::alloc::string::String,
        /// 电话
        #[prost(string, tag = "12")]
        pub tel: ::prost::alloc::string::String,
    }
}
/// 同学基本信息（舍友、可能认识的人或班级同学）
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Student {
    /// 学院
    #[prost(string, tag = "1")]
    pub college: ::prost::alloc::string::String,
    /// 专业
    #[prost(string, tag = "2")]
    pub major: ::prost::alloc::string::String,
    /// 姓名
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    /// 寝室楼号
    #[prost(string, tag = "5")]
    pub building: ::prost::alloc::string::String,
    /// 宿舍门号
    #[prost(int32, tag = "6")]
    pub room: i32,
    /// 床号
    #[prost(int32, tag = "7")]
    pub bed_index: i32,
    /// 性别
    #[prost(enumeration = "super::template::Gender", tag = "8")]
    pub gender: i32,
    /// 该用户上次访问的时间
    #[prost(message, optional, tag = "9")]
    pub last_seen: ::core::option::Option<::prost_types::Timestamp>,
    /// 用户联系方式
    #[prost(string, tag = "10")]
    pub contact: ::prost::alloc::string::String,
    /// 所在省份，研究生、专升本、专科数据可能不全
    #[prost(string, optional, tag = "4")]
    pub province: ::core::option::Option<::prost::alloc::string::String>,
    /// 所在城市，研究生、专升本、专科数据可能不全。
    /// 且某些非直辖市的数据可能包含 `xx 市 xx 区`，长度较长
    #[prost(string, optional, tag = "11")]
    pub city: ::core::option::Option<::prost::alloc::string::String>,
}
/// 我的 “新生数据” 分析
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PersonalAnalysisResult {
    /// 该届与我同名的人数
    #[prost(uint32, tag = "1")]
    pub same_name: u32,
    /// 该届与我来自同一城市（区）
    #[prost(uint32, tag = "2")]
    pub same_city: u32,
    /// 来自同一高中的人数
    #[prost(uint32, tag = "3")]
    pub same_high_school: u32,
    /// 该届学院总人数
    #[prost(uint32, tag = "4")]
    pub college_count: u32,
}
/// Nested message and enum types in `PersonalAnalysisResult`.
pub mod personal_analysis_result {
    /// 该届专业人数情况
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Major {
        /// 专业总人数
        #[prost(int32, tag = "5")]
        pub total: i32,
        /// 男生人数
        #[prost(int32, tag = "6")]
        pub boy: i32,
        /// 女生人数
        #[prost(int32, tag = "7")]
        pub girl: i32,
    }
}
/// “迎新” 模块登录凭证
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FreshmanCredential {
    /// 账户（姓名、学号或准考证号）
    /// 部分用户信息不包含准考证号，但一定包含前两项
    #[prost(string, tag = "2")]
    pub account: ::prost::alloc::string::String,
    /// 用户认证验证码
    /// 一般用身份证后 6 位，也可能使用倒数 2-7 位，数据处理时决定
    #[prost(string, tag = "5")]
    pub secret: ::prost::alloc::string::String,
    /// 入学年份
    #[prost(string, optional, tag = "6")]
    pub entrance_year: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StudentList {
    #[prost(message, repeated, tag = "1")]
    pub student_list: ::prost::alloc::vec::Vec<Student>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FreshmanToken {
    /// 用户通用 JWT 信息
    #[prost(message, optional, tag = "1")]
    pub token: ::core::option::Option<super::token::UserToken>,
    /// 入学年份
    #[prost(string, optional, tag = "2")]
    pub entrance_year: ::core::option::Option<::prost::alloc::string::String>,
}
/// Generated client implementations.
pub mod welcome_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct WelcomeServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl WelcomeServiceClient<tonic::transport::Channel> {
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
    impl<T> WelcomeServiceClient<T>
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
        ) -> WelcomeServiceClient<InterceptedService<T, F>>
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
            WelcomeServiceClient::new(InterceptedService::new(inner, interceptor))
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
        /// 类似登录操作，成功后返回一个通用 JWT 凭据
        pub async fn check_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanCredential>,
        ) -> Result<tonic::Response<super::super::token::UserToken>, tonic::Status> {
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
                "/freshman.WelcomeService/CheckCredential",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取个人相关信息
        pub async fn query_my_self(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::MySelf>, tonic::Status> {
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
                "/freshman.WelcomeService/QueryMySelf",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取舍友列表
        pub async fn get_roommates(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status> {
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
                "/freshman.WelcomeService/GetRoommates",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取同班同学列表
        pub async fn get_classmates(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status> {
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
                "/freshman.WelcomeService/GetClassmates",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取 “可能认识的人” 列表
        pub async fn get_people_may_know(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status> {
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
                "/freshman.WelcomeService/GetPeopleMayKnow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 获取个人数据分析（美名曰“新生大数据”），详见 `PersonalAnalysisResult`
        pub async fn get_personal_analysis(
            &mut self,
            request: impl tonic::IntoRequest<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::PersonalAnalysisResult>, tonic::Status> {
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
                "/freshman.WelcomeService/GetPersonalAnalysis",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod welcome_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with WelcomeServiceServer.
    #[async_trait]
    pub trait WelcomeService: Send + Sync + 'static {
        /// 类似登录操作，成功后返回一个通用 JWT 凭据
        async fn check_credential(
            &self,
            request: tonic::Request<super::FreshmanCredential>,
        ) -> Result<tonic::Response<super::super::token::UserToken>, tonic::Status>;
        /// 获取个人相关信息
        async fn query_my_self(
            &self,
            request: tonic::Request<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::MySelf>, tonic::Status>;
        /// 获取舍友列表
        async fn get_roommates(
            &self,
            request: tonic::Request<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status>;
        /// 获取同班同学列表
        async fn get_classmates(
            &self,
            request: tonic::Request<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status>;
        /// 获取 “可能认识的人” 列表
        async fn get_people_may_know(
            &self,
            request: tonic::Request<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::StudentList>, tonic::Status>;
        /// 获取个人数据分析（美名曰“新生大数据”），详见 `PersonalAnalysisResult`
        async fn get_personal_analysis(
            &self,
            request: tonic::Request<super::FreshmanToken>,
        ) -> Result<tonic::Response<super::PersonalAnalysisResult>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct WelcomeServiceServer<T: WelcomeService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: WelcomeService> WelcomeServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for WelcomeServiceServer<T>
    where
        T: WelcomeService,
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
                "/freshman.WelcomeService/CheckCredential" => {
                    #[allow(non_camel_case_types)]
                    struct CheckCredentialSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanCredential>
                    for CheckCredentialSvc<T> {
                        type Response = super::super::token::UserToken;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanCredential>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).check_credential(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CheckCredentialSvc(inner);
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
                "/freshman.WelcomeService/QueryMySelf" => {
                    #[allow(non_camel_case_types)]
                    struct QueryMySelfSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanToken>
                    for QueryMySelfSvc<T> {
                        type Response = super::MySelf;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).query_my_self(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QueryMySelfSvc(inner);
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
                "/freshman.WelcomeService/GetRoommates" => {
                    #[allow(non_camel_case_types)]
                    struct GetRoommatesSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanToken>
                    for GetRoommatesSvc<T> {
                        type Response = super::StudentList;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_roommates(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetRoommatesSvc(inner);
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
                "/freshman.WelcomeService/GetClassmates" => {
                    #[allow(non_camel_case_types)]
                    struct GetClassmatesSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanToken>
                    for GetClassmatesSvc<T> {
                        type Response = super::StudentList;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_classmates(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetClassmatesSvc(inner);
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
                "/freshman.WelcomeService/GetPeopleMayKnow" => {
                    #[allow(non_camel_case_types)]
                    struct GetPeopleMayKnowSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanToken>
                    for GetPeopleMayKnowSvc<T> {
                        type Response = super::StudentList;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_people_may_know(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPeopleMayKnowSvc(inner);
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
                "/freshman.WelcomeService/GetPersonalAnalysis" => {
                    #[allow(non_camel_case_types)]
                    struct GetPersonalAnalysisSvc<T: WelcomeService>(pub Arc<T>);
                    impl<
                        T: WelcomeService,
                    > tonic::server::UnaryService<super::FreshmanToken>
                    for GetPersonalAnalysisSvc<T> {
                        type Response = super::PersonalAnalysisResult;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreshmanToken>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_personal_analysis(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPersonalAnalysisSvc(inner);
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
    impl<T: WelcomeService> Clone for WelcomeServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: WelcomeService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: WelcomeService> tonic::server::NamedService for WelcomeServiceServer<T> {
        const NAME: &'static str = "freshman.WelcomeService";
    }
}
