//! Pipelines API implementation

use super::types::{ExecutePipelineRequest, ExecutePipelineResponse, PipelineExecutionStatus};
use crate::client::Client;
use crate::error::{self, Error, Result};

/// Pipelines API
pub struct PipelinesApi<'a> {
    client: &'a Client,
}

impl<'a> PipelinesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Execute a pipeline
    pub async fn execute(
        &self,
        request: &ExecutePipelineRequest,
    ) -> Result<ExecutePipelineResponse> {
        let url = format!("{}/v1/pipelines/execute", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 402 {
            Err(error::insufficient_credits())
        } else if response.status() == 429 {
            Err(Error::rate_limited(None))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get pipeline execution status
    pub async fn status(&self, pipeline_execution_id: &str) -> Result<PipelineExecutionStatus> {
        let url = format!(
            "{}/v1/pipelines/executions/{}/status",
            self.client.base_url(),
            pipeline_execution_id
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Pipeline execution {pipeline_execution_id}"
            )))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
