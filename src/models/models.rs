use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeExecutionRequest {
    pub code: String,
    pub language: Language,
    pub timeout: Option<u64>,
    pub stdin: Option<String>,
}

impl CodeExecutionRequest {
    pub fn default() -> CodeExecutionRequest {
        Self {
            code: "".to_string(),
            language: Language::Python,
            timeout: None,
            stdin: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Language {
    Python,
    JavaScript,
    Golang,
    Kotlin,
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" => Ok(Language::Python),
            "javascript" => Ok(Language::JavaScript),
            "golang" => Ok(Language::Golang),
            "kotlin" => Ok(Language::Kotlin),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeExecutionResponse {
    pub execution_id: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
    pub duration: f64,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExecutionRequest {
    pub language: Language,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpload {
    pub code: String,
}