use crate::mapper::mapper::{build_rq_from_multipart, create_error_response};
use crate::models::models::{CodeExecutionRequest, CodeExecutionResponse, FileExecutionRequest};
use crate::service::docker::DockerManager;
use axum::extract::{Multipart, Query};
use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;

pub fn create_router(docker_manager: Arc<DockerManager>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/execute", post(execute_code_handler))
        .route("/execute/file", post(execute_code_from_file_handler))
        .with_state(docker_manager)
}

async fn health_handler() -> Json<Value> {
    Json(json!({"status": "healthy"}))
}

pub async fn execute_code_handler(
    State(docker_manager): State<Arc<DockerManager>>,
    Json(request): Json<CodeExecutionRequest>,
) -> Json<CodeExecutionResponse> {
    log::info!("Executing code for language: {:?}", request.language);

    match docker_manager.execute_code(&request).await {
        Ok(response) => Json(response),
        Err(e) => {
            log::error!("Code execution failed: {:?}", e);
            Json(create_error_response(e.to_string().as_str()))
        }
    }
}

pub async fn execute_code_from_file_handler(
    State(docker_manager): State<Arc<DockerManager>>,
    Query(params): Query<FileExecutionRequest>,
    multipart: Multipart,
) -> Json<CodeExecutionResponse> {
    log::info!(
        "Executing code from file for language: {:?}",
        params.language
    );

    let mut request = CodeExecutionRequest::default();

    match build_rq_from_multipart(multipart, params.language, params.timeout).await {
        Ok(rq) => {
            request = rq;
        }
        Err(e) => return Json(create_error_response(e))
    }

    match docker_manager.execute_code(&request).await {
        Ok(response) => Json(response),
        Err(e) => {
            log::error!("Code execution from file failed: {:?}", e);
            Json(create_error_response(e.to_string().as_str()))
        }
    }
}
