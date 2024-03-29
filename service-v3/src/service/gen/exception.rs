/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

/// 上报的异常信息，由前端 app 自动生成
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Exception {
    /// 错误基本描述
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
    /// 错误发生的时间（用户本地）
    #[prost(message, optional, tag = "2")]
    pub ts: ::core::option::Option<::prost_types::Timestamp>,
    /// 调用栈
    #[prost(string, tag = "3")]
    pub stack: ::prost::alloc::string::String,
    /// 用户平台，JSON
    /// 注意校验其结构
    #[prost(string, tag = "4")]
    pub platform: ::prost::alloc::string::String,
    /// 其他，JSON
    #[prost(string, tag = "5")]
    pub custom: ::prost::alloc::string::String,
    /// 设备信息， JSON
    #[prost(string, tag = "6")]
    pub device: ::prost::alloc::string::String,
    /// 程序版本信息等，JSON
    #[prost(string, tag = "7")]
    pub application: ::prost::alloc::string::String,
}
/// Generated server implementations.
pub mod exception_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with ExceptionServiceServer.
    #[async_trait]
    pub trait ExceptionService: Send + Sync + 'static {
        /// 上报异常信息
        async fn report_exception(
            &self,
            request: tonic::Request<super::Exception>,
        ) -> Result<tonic::Response<super::super::template::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ExceptionServiceServer<T: ExceptionService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ExceptionService> ExceptionServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ExceptionServiceServer<T>
    where
        T: ExceptionService,
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
                "/exception.ExceptionService/ReportException" => {
                    #[allow(non_camel_case_types)]
                    struct ReportExceptionSvc<T: ExceptionService>(pub Arc<T>);
                    impl<T: ExceptionService> tonic::server::UnaryService<super::Exception> for ReportExceptionSvc<T> {
                        type Response = super::super::template::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Exception>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).report_exception(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReportExceptionSvc(inner);
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
    impl<T: ExceptionService> Clone for ExceptionServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ExceptionService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ExceptionService> tonic::server::NamedService for ExceptionServiceServer<T> {
        const NAME: &'static str = "exception.ExceptionService";
    }
}
