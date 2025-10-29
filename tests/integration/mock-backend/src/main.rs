// T051: Mock gRPC backend implementing grpc.health.v1.Health protocol
// Used for integration testing

use anyhow::Result;
use std::env;
use tonic::{transport::Server, Request, Response, Status};

// gRPC Health Check Protocol types
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckRequest {
    #[prost(string, tag = "1")]
    pub service: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckResponse {
    #[prost(enumeration = "ServingStatus", tag = "1")]
    pub status: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ServingStatus {
    Unknown = 0,
    Serving = 1,
    NotServing = 2,
    ServiceUnknown = 3,
}

pub mod health_server {
    use super::*;

    #[derive(Debug, Default)]
    pub struct HealthService {
        pub status: ServingStatus,
    }

    #[tonic::async_trait]
    impl Health for HealthService {
        async fn check(
            &self,
            request: Request<HealthCheckRequest>,
        ) -> Result<Response<HealthCheckResponse>, Status> {
            let service = &request.into_inner().service;

            tracing::info!(
                service = %service,
                status = ?self.status,
                "Health check request received"
            );

            Ok(Response::new(HealthCheckResponse {
                status: self.status as i32,
            }))
        }
    }

    #[tonic::async_trait]
    pub trait Health: Send + Sync + 'static {
        async fn check(
            &self,
            request: Request<HealthCheckRequest>,
        ) -> Result<Response<HealthCheckResponse>, Status>;
    }

    pub fn add_health_service<T: Health>(
        builder: tonic::transport::server::Router,
        service: T,
    ) -> tonic::transport::server::Router {
        use tonic::codegen::*;

        let service = HealthServiceImpl { inner: service };
        builder.add_service(tonic::server::Grpc::new(service))
    }

    struct HealthServiceImpl<T: Health> {
        inner: T,
    }

    #[tonic::async_trait]
    impl<T: Health> tonic::codegen::Service<http::Request<tonic::body::BoxBody>>
        for HealthServiceImpl<T>
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
        >;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<tonic::body::BoxBody>) -> Self::Future {
            let inner = self.inner.clone();
            Box::pin(async move {
                let codec = tonic::codec::ProstCodec::default();
                let mut grpc = tonic::server::Grpc::new(codec);

                let res = match req.uri().path() {
                    "/grpc.health.v1.Health/Check" => {
                        grpc.unary(inner, req, |svc, req| async move {
                            svc.check(req).await
                        }).await
                    }
                    _ => {
                        return Ok(http::Response::builder()
                            .status(http::StatusCode::NOT_FOUND)
                            .body(tonic::body::empty_body())
                            .unwrap());
                    }
                };

                Ok(res)
            })
        }
    }

    impl<T: Health> Clone for HealthServiceImpl<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    let port = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()?;

    let health_status_str = env::var("HEALTH_STATUS")
        .unwrap_or_else(|_| "SERVING".to_string());

    let health_status = match health_status_str.as_str() {
        "SERVING" => ServingStatus::Serving,
        "NOT_SERVING" => ServingStatus::NotServing,
        "UNKNOWN" => ServingStatus::Unknown,
        "SERVICE_UNKNOWN" => ServingStatus::ServiceUnknown,
        _ => ServingStatus::Serving,
    };

    let addr = format!("0.0.0.0:{}", port).parse()?;

    let health_service = health_server::HealthService {
        status: health_status,
    };

    tracing::info!(
        address = %addr,
        status = ?health_status,
        "Mock gRPC backend starting"
    );

    let router = Server::builder();
    let router = health_server::add_health_service(router, health_service);

    router.serve(addr).await?;

    Ok(())
}
