mod config;
mod service;
mod models;
mod routes;
mod mapper;

use std::sync::Arc;
use crate::config::config::Config;
use crate::routes::routes::create_router;
use crate::service::docker::DockerManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    log::info!("Starting server");

    let config = Config::default();
    let docker_manager = DockerManager::new(config.clone())?;
    let docker_manager = Arc::new(docker_manager);

    log::info!("Docker manager initialized");

    let app = create_router(docker_manager);
    let server_address = format!("{}:{}", config.server.host, config.server.port);

    log::info!("Server starting on {}", server_address);

    let listener = tokio::net::TcpListener::bind(&server_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}