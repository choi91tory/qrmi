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
use anyhow::Context;
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint, Identity};

use proto::qpu_executor_client::QpuExecutorClient;
use proto::ExecutorRequest;

pub(crate) async fn ping(
    endpoint: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<bool> {
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

    let mut client = QpuExecutorClient::new(channel);

    let response = client
        .ping(ExecutorRequest {
            message: "QRMI accessibility check".to_string(),
        })
        .await
        .context("LCCFQ Ping request failed")?
        .into_inner();

    Ok(response.success)
}
