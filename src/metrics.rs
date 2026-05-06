// T114-T122: Prometheus metrics module
// Exposes /metrics endpoint with counters, histograms, and gauges

use crate::config::AgentConfig;
use anyhow::Result;
use once_cell::sync::Lazy;
use prometheus::{
    CounterVec, Encoder, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

// T114: CHECK_REQUESTS_TOTAL counter with result label
pub static CHECK_REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new(
            "check_requests_total",
            "Total number of health check requests",
        ),
        &["result"], // "up" or "down"
    )
    .expect("Failed to create CHECK_REQUESTS_TOTAL metric")
});

// T115: CHECK_ERRORS_TOTAL counter with error_type label
pub static CHECK_ERRORS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new("check_errors_total", "Total number of health check errors"),
        &["error_type"], // "timeout", "unreachable", "protocol_error", etc.
    )
    .expect("Failed to create CHECK_ERRORS_TOTAL metric")
});

// T116: CHECK_DURATION_SECONDS histogram with specific buckets
pub static CHECK_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    Histogram::with_opts(
        HistogramOpts::new("check_duration_seconds", "Health check duration in seconds")
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0]),
    )
    .expect("Failed to create CHECK_DURATION_SECONDS metric")
});

// T117: HAPROXY_CONNECTIONS_ACTIVE gauge
pub static HAPROXY_CONNECTIONS_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new(
        "haproxy_connections_active",
        "Number of active HAProxy connections",
    )
    .expect("Failed to create HAPROXY_CONNECTIONS_ACTIVE metric")
});

// T118: GRPC_CHANNELS_ACTIVE gauge
pub static GRPC_CHANNELS_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new(
        "grpc_channels_active",
        "Number of active gRPC channels in cache",
    )
    .expect("Failed to create GRPC_CHANNELS_ACTIVE metric")
});

// T119: Register all metrics
fn register_metrics(registry: &Registry) -> Result<()> {
    registry.register(Box::new(CHECK_REQUESTS_TOTAL.clone()))?;
    registry.register(Box::new(CHECK_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(CHECK_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(HAPROXY_CONNECTIONS_ACTIVE.clone()))?;
    registry.register(Box::new(GRPC_CHANNELS_ACTIVE.clone()))?;
    Ok(())
}

// T120-T122: HTTP server for /metrics endpoint
pub struct MetricsServer {
    registry: Arc<Registry>,
    bind_addr: SocketAddr,
}

impl MetricsServer {
    pub fn new(config: &AgentConfig) -> Result<Self> {
        let registry = Registry::new();
        register_metrics(&registry)?;

        let bind_addr = format!("{}:{}", config.server_bind_address, config.metrics_port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid metrics bind address: {}", e))?;

        Ok(MetricsServer {
            registry: Arc::new(registry),
            bind_addr,
        })
    }

    // T120-T121: Start HTTP server
    pub async fn run(&self) -> Result<()> {
        use hyper::Request;
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;

        // T122: Bind to configured metrics port
        let listener = TcpListener::bind(&self.bind_addr).await.map_err(|e| {
            anyhow::anyhow!("Failed to bind metrics server to {}: {}", self.bind_addr, e)
        })?;

        tracing::debug!(
            address = %self.bind_addr,
            "Metrics server listening"
        );

        let registry = Arc::clone(&self.registry);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let registry = Arc::clone(&registry);

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let registry = Arc::clone(&registry);
                    async move { handle_metrics_request(req, registry).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    tracing::error!(error = %err, "Error serving metrics connection");
                }
            });
        }
    }
}

// T121: Metrics handler returning Prometheus text format
async fn handle_metrics_request(
    req: hyper::Request<hyper::body::Incoming>,
    registry: Arc<Registry>,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, hyper::Error> {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::{Response, StatusCode};

    // Only respond to GET /metrics
    if req.uri().path() != "/metrics" {
        let mut response = Response::new(Full::new(Bytes::from("Not Found")));
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    if req.method() != hyper::Method::GET {
        let mut response = Response::new(Full::new(Bytes::from("Method Not Allowed")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    // Gather metrics
    let metric_families = registry.gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!(error = %e, "Failed to encode metrics");
        let mut response = Response::new(Full::new(Bytes::from("Internal Server Error")));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(response);
    }

    let mut response = Response::new(Full::new(Bytes::from(buffer)));
    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        "text/plain; version=0.0.4; charset=utf-8".parse().unwrap(),
    );
    Ok(response)
}
