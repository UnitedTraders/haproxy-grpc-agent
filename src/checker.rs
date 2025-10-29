// gRPC health checker module
// T057-T066: Complete gRPC health checking implementation

use crate::config::AgentConfig;
use crate::protocol::{HealthCheckRequest, HealthCheckResponse, HealthStatus, SslFlag};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic_prost::ProstCodec;

// T057: BackendChannelKey struct
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendChannelKey {
    pub server: String,
    pub port: u16,
    pub ssl_flag: SslFlag,
}

// T058: From<&HealthCheckRequest> for BackendChannelKey
impl From<&HealthCheckRequest> for BackendChannelKey {
    fn from(req: &HealthCheckRequest) -> Self {
        BackendChannelKey {
            server: req.backend_server.clone(),
            port: req.backend_port,
            ssl_flag: req.ssl_flag,
        }
    }
}

// T059: ChannelCache using DashMap
pub struct GrpcHealthChecker {
    channel_cache: Arc<DashMap<BackendChannelKey, Channel>>,
    config: AgentConfig,
}

impl GrpcHealthChecker {
    pub fn new(config: AgentConfig) -> Self {
        GrpcHealthChecker {
            channel_cache: Arc::new(DashMap::new()),
            config,
        }
    }

    // T060-T061: get_or_create_channel with TLS configuration
    async fn get_or_create_channel(
        &self,
        key: &BackendChannelKey,
        proxy_host: &str,
    ) -> Result<Channel, anyhow::Error> {
        // Check if channel exists in cache
        if let Some(channel) = self.channel_cache.get(key) {
            return Ok(channel.clone());
        }

        // T061: Create new channel with connect timeout
        let endpoint = format!(
            "http{}://{}:{}",
            if matches!(key.ssl_flag, SslFlag::Ssl) {
                "s"
            } else {
                ""
            },
            key.server,
            key.port
        );

        let connect_timeout = Duration::from_millis(self.config.grpc_connect_timeout_ms);

        let mut channel_builder = Channel::from_shared(endpoint.clone())
            .map_err(|e| anyhow::anyhow!("Invalid endpoint {}: {}", endpoint, e))?
            .connect_timeout(connect_timeout);

        // T060: Configure TLS if needed
        if matches!(key.ssl_flag, SslFlag::Ssl) {
            let tls_config = ClientTlsConfig::new().domain_name(proxy_host);
            channel_builder = channel_builder
                .tls_config(tls_config)
                .map_err(|e| anyhow::anyhow!("TLS configuration failed: {}", e))?;
        } else {
            // For non-TLS connections, set the :authority pseudoheader using origin()
            // This is needed for Istio and other service meshes that route based on Host/authority
            let origin_uri = format!("http://{}", proxy_host);
            channel_builder = channel_builder.origin(
                origin_uri
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid origin URI {}: {}", origin_uri, e))?,
            );
        }

        // Connect to backend
        let channel = channel_builder
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("Connection failed to {}: {}", endpoint, e))?;

        // Cache the channel
        self.channel_cache.insert(key.clone(), channel.clone());

        Ok(channel)
    }

    // T063: check_backend function
    pub async fn check_backend(&self, request: &HealthCheckRequest) -> HealthCheckResponse {
        // T066: Error handling - all errors map to Down status
        match self.check_backend_internal(request).await {
            Ok(status) => HealthCheckResponse::new(status),
            Err(e) => {
                tracing::error!(
                    backend = %format!("{}:{}", request.backend_server, request.backend_port),
                    error = %e,
                    "Health check failed"
                );
                HealthCheckResponse::new(HealthStatus::Down)
            }
        }
    }

    async fn check_backend_internal(
        &self,
        request: &HealthCheckRequest,
    ) -> Result<HealthStatus, anyhow::Error> {
        let key = BackendChannelKey::from(request);

        // Get or create channel
        let channel = self
            .get_or_create_channel(&key, &request.proxy_host_name)
            .await?;

        // T062-T064: Create gRPC Health Check client with timeout
        let rpc_timeout = Duration::from_millis(self.config.grpc_rpc_timeout_ms);

        // Import the gRPC health checking protocol types
        use tonic::Request as TonicRequest;

        // Create health check request
        // Note: :authority pseudoheader is set at channel level via origin() or TLS domain_name()
        let health_request = TonicRequest::new(HealthCheckRequestGrpc {
            service: String::new(), // Empty string means overall server health
        });

        // Create client with timeout
        let mut client =
            health_client::HealthClient::new(channel).max_decoding_message_size(usize::MAX);

        // T064: Call with timeout
        let response = tokio::time::timeout(rpc_timeout, client.check(health_request))
            .await
            .map_err(|_| anyhow::anyhow!("Health check RPC timeout after {:?}", rpc_timeout))?
            .map_err(|e| anyhow::anyhow!("Health check RPC failed: {}", e))?;

        // T065: Map ServingStatus to HealthStatus
        let serving_status = response.into_inner().status;
        let health_status = match serving_status {
            0 => HealthStatus::Down, // UNKNOWN
            1 => HealthStatus::Up,   // SERVING
            2 => HealthStatus::Down, // NOT_SERVING
            3 => HealthStatus::Down, // SERVICE_UNKNOWN
            _ => HealthStatus::Down, // Unknown status code
        };

        Ok(health_status)
    }
}

// gRPC Health Check Protocol types
// Based on: https://github.com/grpc/grpc-proto/blob/master/grpc/health/v1/health.proto

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckRequestGrpc {
    #[prost(string, tag = "1")]
    pub service: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HealthCheckResponseGrpc {
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

// gRPC Health service client
pub mod health_client {
    use super::*;
    use tonic::codegen::*;

    #[derive(Debug, Clone)]
    pub struct HealthClient<T> {
        inner: tonic::client::Grpc<T>,
    }

    impl HealthClient<tonic::transport::Channel> {
        pub fn new(channel: tonic::transport::Channel) -> Self {
            let inner = tonic::client::Grpc::new(channel);
            Self { inner }
        }

        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }

        pub async fn check(
            &mut self,
            request: tonic::Request<HealthCheckRequestGrpc>,
        ) -> Result<tonic::Response<HealthCheckResponseGrpc>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e),
                )
            })?;

            let codec = ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/grpc.health.v1.Health/Check");

            self.inner.unary(request, path, codec).await
        }
    }
}
