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

/// 小风筝用户信息
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct User {
    /// uid
    #[prost(int32, tag = "1")]
    pub uid: i32,
    /// 账号，为学生学号，或教师工号。4、9或10位字母或数字
    /// 部分用户可能使用 authserver 的别名功能
    #[prost(string, tag = "2")]
    pub account: ::prost::alloc::string::String,
    /// 账号创建时间
    #[prost(message, optional, tag = "3")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// OA 登录凭据
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OaCredential {
    /// 账号，详见 User.account 描述
    #[prost(string, tag = "1")]
    pub account: ::prost::alloc::string::String,
    /// OA 密码
    #[prost(string, tag = "2")]
    pub password: ::prost::alloc::string::String,
}
/// 登录过程， client -> kite-server 流数据
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientStream {
    #[prost(oneof = "client_stream::Payload", tags = "1, 2")]
    pub payload: ::core::option::Option<client_stream::Payload>,
}
/// Nested message and enum types in `ClientStream`.
pub mod client_stream {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        /// OA 凭据
        #[prost(message, tag = "1")]
        Credential(super::OaCredential),
        /// 来自 authserver 的 TLS 流数据，经由 client 转发到 kite-server
        #[prost(bytes, tag = "2")]
        TlsStream(::prost::alloc::vec::Vec<u8>),
    }
}
/// 登录过程， kite-server -> app
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerStream {
    #[prost(oneof = "server_stream::Payload", tags = "1, 2, 3")]
    pub payload: ::core::option::Option<server_stream::Payload>,
}
/// Nested message and enum types in `ServerStream`.
pub mod server_stream {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        /// 用户登录成功凭据
        #[prost(message, tag = "1")]
        User(super::User),
        /// 来自 kite-server 的数据，经由 client 发往 authserver 的流数据
        #[prost(bytes, tag = "2")]
        TlsStream(::prost::alloc::vec::Vec<u8>),
        /// 用户登录失败的错误提示
        #[prost(string, tag = "3")]
        Message(::prost::alloc::string::String),
    }
}
/// Generated server implementations.
pub mod user_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]

    use tonic::codegen::*;

    /// Generated trait containing gRPC methods that should be implemented for use with UserServiceServer.
    #[async_trait]
    pub trait UserService: Send + Sync + 'static {
        /// Server streaming response type for the Login method.
        type LoginStream: futures_core::Stream<Item = Result<super::ServerStream, tonic::Status>> + Send + 'static;
        /// 登录小风筝账户
        ///
        /// 受限于若干网络上的限制，需要使用用户侧手机作为 socks5 代理使用。该登录方案的原理是，建立一条 kite-server 和
        /// authserver.sit.edu.cn 之间的 TLS 连接，以确保通信不被用户（也可能是潜在的攻击者）监听和篡改。
        /// 该方案保证 server 可以可靠地验证用户提供的用户名和密码，同时避免了 IP 重试次数过多被防火墙封禁。
        async fn login(
            &self,
            request: tonic::Request<tonic::Streaming<super::ClientStream>>,
        ) -> Result<tonic::Response<Self::LoginStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct UserServiceServer<T: UserService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: UserService> UserServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for UserServiceServer<T>
    where
        T: UserService,
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
                "/user.UserService/Login" => {
                    #[allow(non_camel_case_types)]
                    struct LoginSvc<T: UserService>(pub Arc<T>);
                    impl<T: UserService> tonic::server::StreamingService<super::ClientStream> for LoginSvc<T> {
                        type Response = super::ServerStream;
                        type ResponseStream = T::LoginStream;
                        type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::ClientStream>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).login(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoginSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(accept_compression_encodings, send_compression_encodings);
                        let res = grpc.streaming(method, req).await;
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
    impl<T: UserService> Clone for UserServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: UserService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: UserService> tonic::server::NamedService for UserServiceServer<T> {
        const NAME: &'static str = "user.UserService";
    }
}
