// T051: Mock gRPC backend implementing grpc.health.v1.Health protocol
// Used for integration testing

use anyhow::Result;
use std::env;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tonic_health::ServingStatus;

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

    let addr = format!("0.0.0.0:{}", port).parse()?;

    let (reporter, health_service) = health_reporter();

    // Set overall service health status
    match health_status_str.as_str() {
        "SERVING" => reporter
            .set_service_status("", ServingStatus::Serving)
            .await,
        "NOT_SERVING" => reporter
            .set_service_status("", ServingStatus::NotServing)
            .await,
        "UNKNOWN" => reporter
            .set_service_status("", ServingStatus::Unknown)
            .await,
        "SERVICE_UNKNOWN" => reporter
            .set_service_status("", ServingStatus::Unknown)
            .await,
        _ => reporter
            .set_service_status("", ServingStatus::Serving)
            .await,
    };

    tracing::info!(
        address = %addr,
        status = %health_status_str,
        "Mock gRPC backend starting"
    );

    Server::builder()
        .add_service(health_service)
        .serve(addr)
        .await?;

    Ok(())
}
