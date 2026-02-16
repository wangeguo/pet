use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MESHY_API_BASE: &str = "https://api.meshy.ai/v2";

pub struct MeshyClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct CreateTaskRequest {
    mode: String,
    prompt: String,
    art_style: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTaskResponse {
    pub result: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskStatusResponse {
    pub status: String,
    pub model_urls: Option<ModelUrls>,
    pub thumbnail_url: Option<String>,
    pub progress: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelUrls {
    pub glb: Option<String>,
}

#[derive(Error, Debug)]
pub enum MeshyError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),
}

impl MeshyClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn create_task(&self, prompt: &str) -> Result<String, MeshyError> {
        let resp = self
            .client
            .post(format!("{MESHY_API_BASE}/text-to-3d"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&CreateTaskRequest {
                mode: "preview".to_string(),
                prompt: prompt.to_string(),
                art_style: "realistic".to_string(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<CreateTaskResponse>()
            .await?;

        Ok(resp.result)
    }

    pub async fn get_task_status(&self, task_id: &str) -> Result<TaskStatusResponse, MeshyError> {
        let resp = self
            .client
            .get(format!("{MESHY_API_BASE}/text-to-3d/{task_id}"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .error_for_status()?
            .json::<TaskStatusResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, MeshyError> {
        let bytes = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        Ok(bytes.to_vec())
    }
}
