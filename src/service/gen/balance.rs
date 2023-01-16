/// 房间电费余额信息
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoomBalance {
    /// 房间号，规则为 “10” + 楼号 + 房间号（仅限奉贤校区）
    #[prost(int32, tag = "1")]
    pub room: i32,
    /// 电费余额值，为空调电和普通用电余额所加之和
    #[prost(float, tag = "2")]
    pub balance: f32,
    /// 上次更新日期，以该房间变化为准
    #[prost(message, optional, tag = "4")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
}
/// 消费情况
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BillItem {
    /// 电费较上一单位时间增加值，用于统计电费充值情况
    #[prost(float, tag = "3")]
    pub increment: f32,
    /// 电费较上一单位时间减少值
    #[prost(float, tag = "4")]
    pub decrement: f32,
    /// 横坐标信息，判断是天还是小时
    #[prost(oneof = "bill_item::Identifier", tags = "1, 2")]
    pub identifier: ::core::option::Option<bill_item::Identifier>,
}
/// Nested message and enum types in `BillItem`.
pub mod bill_item {
    /// 横坐标信息，判断是天还是小时
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Identifier {
        /// 如：2022-2-1
        #[prost(string, tag = "1")]
        Date(::prost::alloc::string::String),
        /// 如：19 （表示19时）
        #[prost(string, tag = "2")]
        Time(::prost::alloc::string::String),
    }
}
/// 电费使用排名
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumptionRank {
    /// 消费金额值
    #[prost(float, tag = "1")]
    pub consumption: f32,
    /// 总排名
    #[prost(int32, tag = "2")]
    pub rank: i32,
    /// 总房间数
    #[prost(int32, tag = "3")]
    pub total_room: i32,
}
/// 电费余额及排名的请求
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceRequest {
    #[prost(int32, tag = "1")]
    pub room_number: i32,
}
/// 电费使用情况请求
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BillRequest {
    #[prost(int32, tag = "1")]
    pub room_number: i32,
    #[prost(enumeration = "BillType", tag = "2")]
    pub r#type: i32,
}
/// 电费使用情况结果
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BillResponse {
    #[prost(message, repeated, tag = "1")]
    pub bill_list: ::prost::alloc::vec::Vec<BillItem>,
}
/// 统计类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BillType {
    /// 以“日”为单位
    Daily = 0,
    /// 以“小时”为单位
    Hourly = 1,
}
impl BillType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BillType::Daily => "Daily",
            BillType::Hourly => "Hourly",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Daily" => Some(Self::Daily),
            "Hourly" => Some(Self::Hourly),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod balance_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::http::Uri;
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct BalanceServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BalanceServiceClient<tonic::transport::Channel> {
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
    impl<T> BalanceServiceClient<T>
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> BalanceServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            BalanceServiceClient::new(InterceptedService::new(inner, interceptor))
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
        /// 请求单一房间电费余额情况
        pub async fn get_room_balance(
            &mut self,
            request: impl tonic::IntoRequest<super::BalanceRequest>,
        ) -> Result<tonic::Response<super::RoomBalance>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/balance.BalanceService/GetRoomBalance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 请求单一房间电费排名情况
        pub async fn get_consumption_rank(
            &mut self,
            request: impl tonic::IntoRequest<super::BalanceRequest>,
        ) -> Result<tonic::Response<super::ConsumptionRank>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/balance.BalanceService/GetConsumptionRank");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// 按类型请求电费统计情况
        pub async fn get_bill(
            &mut self,
            request: impl tonic::IntoRequest<super::BillRequest>,
        ) -> Result<tonic::Response<super::BillResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/balance.BalanceService/GetBill");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod balance_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with BalanceServiceServer.
    #[async_trait]
    pub trait BalanceService: Send + Sync + 'static {
        /// 请求单一房间电费余额情况
        async fn get_room_balance(
            &self,
            request: tonic::Request<super::BalanceRequest>,
        ) -> Result<tonic::Response<super::RoomBalance>, tonic::Status>;
        /// 请求单一房间电费排名情况
        async fn get_consumption_rank(
            &self,
            request: tonic::Request<super::BalanceRequest>,
        ) -> Result<tonic::Response<super::ConsumptionRank>, tonic::Status>;
        /// 按类型请求电费统计情况
        async fn get_bill(
            &self,
            request: tonic::Request<super::BillRequest>,
        ) -> Result<tonic::Response<super::BillResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct BalanceServiceServer<T: BalanceService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BalanceService> BalanceServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BalanceServiceServer<T>
    where
        T: BalanceService,
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
                "/balance.BalanceService/GetRoomBalance" => {
                    #[allow(non_camel_case_types)]
                    struct GetRoomBalanceSvc<T: BalanceService>(pub Arc<T>);
                    impl<T: BalanceService> tonic::server::UnaryService<super::BalanceRequest> for GetRoomBalanceSvc<T> {
                        type Response = super::RoomBalance;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::BalanceRequest>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_room_balance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetRoomBalanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/balance.BalanceService/GetConsumptionRank" => {
                    #[allow(non_camel_case_types)]
                    struct GetConsumptionRankSvc<T: BalanceService>(pub Arc<T>);
                    impl<T: BalanceService> tonic::server::UnaryService<super::BalanceRequest> for GetConsumptionRankSvc<T> {
                        type Response = super::ConsumptionRank;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::BalanceRequest>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_consumption_rank(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetConsumptionRankSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/balance.BalanceService/GetBill" => {
                    #[allow(non_camel_case_types)]
                    struct GetBillSvc<T: BalanceService>(pub Arc<T>);
                    impl<T: BalanceService> tonic::server::UnaryService<super::BillRequest> for GetBillSvc<T> {
                        type Response = super::BillResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::BillRequest>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_bill(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBillSvc(inner);
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
    impl<T: BalanceService> Clone for BalanceServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: BalanceService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BalanceService> tonic::server::NamedService for BalanceServiceServer<T> {
        const NAME: &'static str = "balance.BalanceService";
    }
}
