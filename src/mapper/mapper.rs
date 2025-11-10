use axum::extract::Multipart;
use crate::models::models::{CodeExecutionRequest, CodeExecutionResponse, Language};

pub async fn build_rq_from_multipart(mut multipart: Multipart, language: Language, timeout: Option<u64>) -> Result<CodeExecutionRequest, &'static str> {
    let mut code_content = String::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "code" {
            if let Ok(content) = field.text().await {
                code_content = content;
                break;
            }
        }
    }

    if code_content.is_empty() {
        return Err("code is empty")
    }

    Ok(CodeExecutionRequest {
        language,
        code: code_content,
        timeout,
        stdin: None,
    })
}

pub fn create_error_response(error_message: &str) -> CodeExecutionResponse {
    CodeExecutionResponse {
        execution_id: "error".to_string(),
        stdout: String::new(),
        stderr: error_message.to_string(),
        exit_code: -1,
        duration: 0.0,
        timed_out: false,
    }
}