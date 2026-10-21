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

use crate::Result;
use serde::Deserialize;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

use anyhow::{anyhow, Context};
use proto::qpu_executor_client::QpuExecutorClient;
use proto::{ExecutorRequest, Gate, SubmitCircuitTaskRequest, SubmitTestTaskRequest};
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
