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

use super::LCCFQServer;
use crate::models::{Payload, ResourceType};
use crate::{QrmiError, QuantumResource};

#[tokio::test]
async fn resource_id_type_and_metadata_match_configuration() {
    const RESOURCE_ID: &str = "lccfq-qpu";
    const ENDPOINT: &str = "https://localhost:50052";

    let mut qrmi = LCCFQServer {
        resource_id: RESOURCE_ID.to_string(),
        endpoint: ENDPOINT.to_string(),
        ca_cert_path: "unused-ca.crt".to_string(),
        client_cert_path: "unused-client.crt".to_string(),
        client_key_path: "unused-client.key".to_string(),
    };

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");

    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, RESOURCE_ID);
    assert_eq!(resource_type, ResourceType::LCCFQ);

    let metadata = qrmi.metadata().await;

    assert_eq!(metadata["resource_id"], RESOURCE_ID);
    assert_eq!(metadata["resource_type"], "lccfq");
    assert_eq!(metadata["backend_endpoint"], ENDPOINT);
}

#[tokio::test]
async fn task_start_rejects_unknown_task_type() {
    let mut qrmi = LCCFQServer {
        resource_id: "lccfq-qpu".to_string(),
        endpoint: "https://localhost:50052".to_string(),
        ca_cert_path: "unused-ca.crt".to_string(),
        client_cert_path: "unused-client.crt".to_string(),
        client_key_path: "unused-client.key".to_string(),
    };

    let payload = Payload::LCCFQ {
        input: "{}".to_string(),
        task_type: "unknown".to_string(),
    };

    let error = qrmi
        .task_start(payload)
        .await
        .expect_err("unknown task type should fail");

    assert!(matches!(error, QrmiError::InvalidInput(_)));
}

#[tokio::test]
async fn task_start_rejects_invalid_json() {
    let mut qrmi = LCCFQServer {
        resource_id: "lccfq-qpu".to_string(),
        endpoint: "https://localhost:50052".to_string(),
        ca_cert_path: "unused-ca.crt".to_string(),
        client_cert_path: "unused-client.crt".to_string(),
        client_key_path: "unused-client.key".to_string(),
    };

    let payload = Payload::LCCFQ {
        input: "not valid JSON".to_string(),
        task_type: "circuit".to_string(),
    };

    let error = qrmi
        .task_start(payload)
        .await
        .expect_err("invalid JSON should fail");

    assert!(matches!(error, QrmiError::InvalidJson(_)));
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn submit_circuit_task_with_running_backend() {
    let mut qrmi = LCCFQServer::new("lccfq_qpu").expect("LCCFQServer should be created");

    let payload = Payload::LCCFQ {
        input: r#"
        {
            "gates": [
                {
                    "symbol": "H",
                    "target_qubits": [0],
                    "control_qubits": [],
                    "params": []
                }
            ],
            "shots": 10
        }
        "#
        .to_string(),
        task_type: "circuit".to_string(),
    };

    let task_id = qrmi
        .task_start(payload)
        .await
        .expect("circuit task submission should succeed");

    assert!(!task_id.is_empty());
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn submit_test_task_with_running_backend() {
    let mut qrmi = LCCFQServer::new("lccfq_qpu").expect("LCCFQServer should be created");

    let payload = Payload::LCCFQ {
        input: r#"
        {
            "symbol": "XEB",
            "params": [1, 2],
            "shots": 10
        }
        "#
        .to_string(),
        task_type: "test".to_string(),
    };

    let task_id = qrmi
        .task_start(payload)
        .await
        .expect("test task submission should succeed");

    assert!(!task_id.is_empty());
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn is_accessible_with_running_backend() {
    let mut qrmi = LCCFQServer::new("lccfq_qpu").expect("LCCFQServer should be created");

    let accessible = qrmi
        .is_accessible()
        .await
        .expect("LCCFQ Ping should succeed");

    assert!(accessible);
}
