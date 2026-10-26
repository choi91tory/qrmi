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

// Names in this module are generated from the LCCFQ protobuf definition.
#[allow(clippy::enum_variant_names)]
pub(crate) mod proto {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

use crate::models::{TaskResult, TaskStatus};
use crate::{QrmiError, Result};
use serde::Deserialize;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

use anyhow::{anyhow, Context};
use proto::get_result_response::Result as ResultPayload;
use proto::qpu_executor_client::QpuExecutorClient;
use proto::{
    CancelTaskRequest, ExecutorRequest, Gate, GetResultRequest, GetTaskStatusRequest,
    SubmitCircuitTaskRequest, SubmitTestTaskRequest, TaskStatusValue,
};

#[derive(Debug, Deserialize)]
struct CircuitTaskInput {
    gates: Vec<CircuitGateInput>,
    shots: i32,
}

#[derive(Debug, Deserialize)]
struct CircuitGateInput {
    symbol: String,
    target_qubits: Vec<i32>,
    control_qubits: Vec<i32>,
    params: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct TestTaskInput {
    symbol: String,
    params: Vec<i32>,
    shots: i32,
}

pub(crate) async fn connect(
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<QpuExecutorClient<Channel>> {
    let ca_cert = tokio::fs::read(ca_cert_path)
        .await
        .context("failed to read LCCFQ CA certificate")?;

    let client_cert = tokio::fs::read(client_cert_path)
        .await
        .context("failed to read LCCFQ client certificate")?;

    let client_key = tokio::fs::read(client_key_path)
        .await
        .context("failed to read LCCFQ client key")?;

    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca_cert))
        .identity(Identity::from_pem(client_cert, client_key));

    let channel = Endpoint::from_shared(endpoint.to_string())
        .context("invalid LCCFQ endpoint")?
        .tls_config(tls)
        .context("failed to configure LCCFQ TLS")?
        .connect()
        .await
        .context("failed to connect to LCCFQ Backend")?;

    Ok(QpuExecutorClient::new(channel))
}

pub(crate) async fn ping(
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<bool> {
    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .ping(ExecutorRequest {
            message: "QRMI accessibility check".to_string(),
        })
        .await
        .context("LCCFQ Ping request failed")?
        .into_inner();

    Ok(response.success)
}

pub(crate) async fn submit_circuit_task(
    input: &str,
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<String> {
    let input: CircuitTaskInput = serde_json::from_str(input)?;

    let gates = input
        .gates
        .into_iter()
        .map(|gate| Gate {
            symbol: gate.symbol,
            target_qubits: gate.target_qubits,
            control_qubits: gate.control_qubits,
            params: gate.params,
        })
        .collect();

    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .submit_circuit_task(SubmitCircuitTaskRequest {
            gates,
            shots: input.shots,
        })
        .await
        .context("LCCFQ SubmitCircuitTask request failed")?
        .into_inner();

    if !response.success {
        return Err(anyhow!("LCCFQ circuit task submission failed: {}", response.message).into());
    }

    Ok(response.task_id)
}

pub(crate) async fn submit_test_task(
    input: &str,
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<String> {
    let input: TestTaskInput = serde_json::from_str(input)?;

    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .submit_test_task(SubmitTestTaskRequest {
            symbol: input.symbol,
            params: input.params,
            shots: input.shots,
        })
        .await
        .context("LCCFQ SubmitTestTask request failed")?
        .into_inner();

    if !response.success {
        return Err(anyhow!("LCCFQ test task submission failed: {}", response.message).into());
    }

    Ok(response.task_id)
}

pub(crate) async fn get_result(
    task_id: &str,
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<TaskResult> {
    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .get_result(GetResultRequest {
            task_id: task_id.to_string(),
        })
        .await
        .context("LCCFQ GetResult request failed")?
        .into_inner();

    if !response.found {
        return Err(QrmiError::TaskNotReady {
            task_id: task_id.to_string(),
            reason: response.error_message,
        });
    }

    let result = match response.result {
        Some(ResultPayload::CircuitResult(result)) => serde_json::json!({
            "type": "circuit",
            "task_id": result.task_id,
            "distribution": result.distribution,
        }),

        Some(ResultPayload::TestResult(result)) => serde_json::json!({
            "type": "test",
            "task_id": result.task_id,
            "parameters": result.parameters,
        }),

        Some(ResultPayload::ControlAck(result)) => serde_json::json!({
            "type": "control",
            "task_id": result.task_id,
            "status": result.status,
            "message": result.message,
        }),

        None => {
            return Err(
                anyhow!("LCCFQ GetResult response did not contain a result payload").into(),
            );
        }
    };

    Ok(TaskResult {
        value: serde_json::to_string(&result)?,
    })
}

pub(crate) async fn get_task_status(
    task_id: &str,
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<TaskStatus> {
    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .get_task_status(GetTaskStatusRequest {
            task_id: task_id.to_string(),
        })
        .await
        .context("LCCFQ GetTaskStatus request failed")?
        .into_inner();

    if !response.found {
        return Err(QrmiError::TaskNotFound(task_id.to_string()));
    }

    let status = TaskStatusValue::try_from(response.status)
        .context("LCCFQ GetTaskStatus returned an unknown status value")?;

    match status {
        TaskStatusValue::TaskStatusQueued => Ok(TaskStatus::Queued),
        TaskStatusValue::TaskStatusRunning => Ok(TaskStatus::Running),
        TaskStatusValue::TaskStatusCompleted => Ok(TaskStatus::Completed),
        TaskStatusValue::TaskStatusFailed => Ok(TaskStatus::Failed),
        TaskStatusValue::TaskStatusCancelled => Ok(TaskStatus::Cancelled),
        TaskStatusValue::TaskStatusUnspecified => {
            Err(anyhow!("LCCFQ returned an unspecified task status").into())
        }
    }
}

pub(crate) async fn cancel_task(
    task_id: &str,
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<()> {
    let mut client = connect(endpoint, ca_cert_path, client_cert_path, client_key_path).await?;

    let response = client
        .cancel_task(CancelTaskRequest {
            task_id: task_id.to_string(),
        })
        .await
        .context("LCCFQ CancelTask request failed")?
        .into_inner();

    if !response.found {
        return Err(QrmiError::TaskNotFound(task_id.to_string()));
    }

    if !response.cancelled {
        return Err(QrmiError::InvalidInput(response.error_message));
    }

    Ok(())
}
