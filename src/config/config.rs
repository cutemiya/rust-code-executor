use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub docker: DockerConfig,
    pub execution: ExecutionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DockerConfig {
    pub socket_path: String,
    pub network: String,
    pub memory_limit: String,
    pub cpu_shares: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionConfig {
    pub default_timeout: u64,
    pub max_output_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            docker: DockerConfig {
                socket_path: "/var/run/docker.sock".to_string(),
                network: "none".to_string(),
                memory_limit: "100m".to_string(),
                cpu_shares: 512,
            },
            execution: ExecutionConfig {
                default_timeout: 120,
                max_output_size: 10 * 1024 * 1024,
            },
        }
    }
}