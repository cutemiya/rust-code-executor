use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, WaitContainerOptions},
    Docker,
};
use bollard::query_parameters::KillContainerOptions;
use futures::StreamExt;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use tokio::sync::mpsc;

use crate::config::config::Config as AppConfig;
use crate::models::models::{CodeExecutionRequest, CodeExecutionResponse, Language};

#[derive(Clone)]
pub struct DockerManager {
    sender: mpsc::Sender<DockerCommand>,
}

struct DockerExecutor {
    docker: Docker,
    config: AppConfig,
}

enum DockerCommand {
    ExecuteCode {
        request: CodeExecutionRequest,
        respond_to: mpsc::Sender<Result<CodeExecutionResponse, String>>,
    },
}

impl DockerManager {
    pub fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, mut receiver) = mpsc::channel(32);
        let executor = DockerExecutor::new(config)?;

        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                executor.handle_command(command).await;
            }
        });

        Ok(Self { sender })
    }

    pub async fn execute_code(
        &self,
        request: &CodeExecutionRequest,
    ) -> Result<CodeExecutionResponse, Box<dyn std::error::Error>> {
        let (response_sender, mut response_receiver) = mpsc::channel(1);

        let command = DockerCommand::ExecuteCode {
            request: request.clone(),
            respond_to: response_sender,
        };

        self.sender.send(command).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        response_receiver.recv().await
            .ok_or("No response received")?
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)
    }
}

impl DockerExecutor {
    fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let docker = Docker::connect_with_socket(&config.docker.socket_path, 120, &bollard::API_DEFAULT_VERSION)?;
        Ok(Self { docker, config })
    }

    async fn handle_command(&self, command: DockerCommand) {
        match command {
            DockerCommand::ExecuteCode { request, respond_to } => {
                let result = self.execute_code_impl(&request).await;
                let _ = respond_to.send(result).await;
            }
        }
    }

    async fn execute_code_impl(
        &self,
        request: &CodeExecutionRequest,
    ) -> Result<CodeExecutionResponse, String> {
        let execution_id = Uuid::new_v4();
        let container_name = format!("code_exec_{}", execution_id);

        let timeout_duration = Duration::from_secs(request.timeout.unwrap_or(self.config.execution.default_timeout));

        let container_config = match self.create_container_config(request.language, &request.code) {
            Ok(config) => config,
            Err(e) => return Err(e.to_string()),
        };

        let container = match self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: &container_name,
                    platform: None,
                }),
                container_config,
            )
            .await {
            Ok(container) => container,
            Err(e) => return Err(e.to_string()),
        };

        if let Err(e) = self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await {
            return Err(e.to_string());
        }

        let start_time = std::time::Instant::now();

        let wait_result = timeout(
            timeout_duration,
            self.wait_for_container_completion(&container.id)
        ).await;

        let (exit_code, timed_out) = match wait_result {
            Ok(Ok(code)) => (code, false),
            Ok(Err(e)) => {
                log::error!("Error waiting for container: {}", e);
                (-1, false)
            }
            Err(_) => {
                log::warn!("Container execution timeout, killing container: {}", container.id);
                let options = Some(KillContainerOptions{
                    signal: "SIGINT".to_string(),
                });
                let _ = self.docker.kill_container(&container.id, options).await;
                (-1, true)
            }
        };

        let duration = start_time.elapsed().as_secs_f64();
        let logs = self.get_container_logs(&container.id).await.unwrap_or_default();
        let _ = self.cleanup_container(&container.id).await;

        Ok(CodeExecutionResponse {
            execution_id: execution_id.to_string(),
            stdout: self.truncate_output(logs.stdout),
            stderr: self.truncate_output(logs.stderr),
            exit_code,
            duration,
            timed_out,
        })
    }

    fn create_container_config(
        &self,
        language: Language,
        code: &str,
    ) -> Result<Config<String>, Box<dyn std::error::Error>> {
        let escaped_code = code
            .replace("'", "\"");

        let (image, shell_command) = match language {
            Language::Python => (
                "python:3.9-slim",
                format!("mkdir -p /app && echo '{}' > /app/code.py && python /app/code.py", escaped_code)
            ),
            Language::JavaScript => (
                "node:18-alpine",
                format!("mkdir -p /app && echo '{}' > /app/code.js && node /app/code.js", escaped_code)
            ),
            Language::Golang => (
                "golang:1.19-alpine",
                format!("mkdir -p /app && echo '{}' > /app/code.go && cd /app && go run code.go", escaped_code)
            ),
            Language::Kotlin => (
                "kotlin:latest",
                format!("mkdir -p /app && echo '{}' > /app/code.kt && cd /app && kotlinc code.kt -include-runtime -d code.jar && java -jar code.jar", escaped_code)
            ),
        };

        let config = Config {
            image: Some(image.to_string()),
            cmd: Some(vec!["sh".to_string(), "-c".to_string(), shell_command]),
            host_config: Some(bollard::models::HostConfig {
                network_mode: Some(self.config.docker.network.clone()),
                memory: Some(self.config.docker.memory_limit.parse().unwrap_or(100 * 1024 * 1024)),
                memory_swap: Some(0),
                cpu_shares: Some(self.config.docker.cpu_shares),
                auto_remove: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(config)
    }

    async fn wait_for_container_completion(
        &self,
        container_id: &str,
    ) -> Result<i64, String> {
        let options = Some(WaitContainerOptions {
            condition: "not-running",
        });

        let mut stream = self.docker.wait_container(container_id, options);

        while let Some(result) = stream.next().await {
            return match result {
                Ok(wait_response) => Ok(wait_response.status_code),
                Err(e) => Err(e.to_string())
            }
        }

        Err("Container stream ended unexpectedly".into())
    }

    async fn get_container_logs(
        &self,
        container_id: &str,
    ) -> Result<ContainerLogs, String> {
        let options = Some(bollard::container::LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: false,
            follow: false,
            tail: "all",
            ..Default::default()
        });

        let mut logs_stream = self.docker.logs(container_id, options);

        let mut stdout = String::new();
        let mut stderr = String::new();

        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(log_output) => {
                    match log_output {
                        bollard::container::LogOutput::StdOut { message } => {
                            if let Ok(text) = String::from_utf8(message.to_vec()) {
                                stdout.push_str(&text);
                            }
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            if let Ok(text) = String::from_utf8(message.to_vec()) {
                                stderr.push_str(&text);
                            }
                        }
                        bollard::container::LogOutput::Console { message } => {
                            if let Ok(text) = String::from_utf8(message.to_vec()) {
                                stdout.push_str(&text);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    log::warn!("Error reading logs: {}", e);
                }
            }
        }

        Ok(ContainerLogs { stdout, stderr })
    }

    async fn cleanup_container(&self, container_id: &str) -> Result<(), String> {
        let options = Some(RemoveContainerOptions {
            force: true,
            v: true,
            ..Default::default()
        });

        match self.docker.remove_container(container_id, options).await {
            Ok(_) => {
                log::debug!("Container {} removed successfully", container_id);
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to remove container {}: {}", container_id, e);
                Ok(())
            }
        }
    }

    fn truncate_output(&self, output: String) -> String {
        if output.len() > self.config.execution.max_output_size {
            let truncated: String = output.chars().take(self.config.execution.max_output_size).collect();
            format!("{}...\n[Output truncated]", truncated)
        } else {
            output
        }
    }
}

#[derive(Debug, Default)]
struct ContainerLogs {
    stdout: String,
    stderr: String,
}