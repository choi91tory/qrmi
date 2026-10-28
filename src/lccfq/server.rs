// This code is part of Qiskit.
//
// (C) Copyright NCSA 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::client;
use crate::error::required_env;
use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::{QrmiError, QuantumResource, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// QRMI implementation for LCCFQ Backend.
pub struct LCCFQServer {
    pub(crate) resource_id: String,
    pub(crate) endpoint: String,
    pub(crate) ca_cert_path: String,
    pub(crate) client_cert_path: String,
    pub(crate) client_key_path: String,
}

impl LCCFQServer {
    /// Creates an LCCFQ resource.
    pub fn new(resource_id: &str) -> Result<Self> {
        Ok(Self {
            resource_id: resource_id.to_string(),
            endpoint: required_env(format!("{resource_id}_QRMI_LCCFQ_ENDPOINT"))?,
            ca_cert_path: required_env(format!("{resource_id}_QRMI_LCCFQ_CA_CERT"))?,
            client_cert_path: required_env(format!("{resource_id}_QRMI_LCCFQ_CLIENT_CERT"))?,
            client_key_path: required_env(format!("{resource_id}_QRMI_LCCFQ_CLIENT_KEY"))?,
        })
    }
}

#[async_trait]
impl QuantumResource for LCCFQServer {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.resource_id.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::LCCFQ)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        client::ping(
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        match payload {
            Payload::LCCFQ { input, task_type } => match task_type.as_str() {
                "circuit" => {
                    client::submit_circuit_task(
                        &input,
                        &self.endpoint,
                        &self.ca_cert_path,
                        &self.client_cert_path,
                        &self.client_key_path,
                    )
                    .await
                }
                "test" => {
                    client::submit_test_task(
                        &input,
                        &self.endpoint,
                        &self.ca_cert_path,
                        &self.client_cert_path,
                        &self.client_key_path,
                    )
                    .await
                }
                _ => Err(QrmiError::InvalidInput(format!(
                    "unsupported LCCFQ task type: {task_type}"
                ))),
            },
            other => Err(QrmiError::UnsupportedPayload(format!("{other:?}"))),
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        client::cancel_task(
            task_id,
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        client::get_task_status(
            task_id,
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        client::get_result(
            task_id,
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        client::get_task_logs(
            task_id,
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn target(&mut self) -> Result<Target> {
        client::get_target(
            &self.endpoint,
            &self.ca_cert_path,
            &self.client_cert_path,
            &self.client_key_path,
        )
        .await
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        metadata.insert("resource_id".to_string(), self.resource_id.clone());

        metadata.insert(
            "resource_type".to_string(),
            ResourceType::LCCFQ.as_str().to_string(),
        );

        metadata.insert("backend_endpoint".to_string(), self.endpoint.clone());

        metadata
    }
}

#[cfg(test)]
#[path = "tests/lccfq_server.rs"]
mod tests;
