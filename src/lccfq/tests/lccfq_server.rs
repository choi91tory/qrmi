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
use crate::models::{Payload, ResourceType, TaskResult, TaskStatus};
use crate::{QrmiError, QuantumResource};
use tokio::time::{sleep, Duration};

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

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn task_result_returns_circuit_result_with_running_backend() {
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

    let mut task_result: Option<TaskResult> = None;

    for _ in 0..30 {
        match qrmi.task_result(&task_id).await {
            Ok(result) => {
                task_result = Some(result);
                break;
            }
            Err(QrmiError::TaskNotReady { .. }) => {
                sleep(Duration::from_millis(500)).await;
            }
            Err(error) => {
                panic!("task result retrieval failed: {error}");
            }
        }
    }

    let task_result = task_result.expect("task result should become available");

    let result_json: serde_json::Value =
        serde_json::from_str(&task_result.value).expect("task result should contain valid JSON");

    assert_eq!(result_json["type"], "circuit");
    assert_eq!(result_json["task_id"], task_id);
    assert!(result_json["distribution"].is_object());
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn task_result_returns_test_result_with_running_backend() {
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

    let mut task_result: Option<TaskResult> = None;

    for _ in 0..30 {
        match qrmi.task_result(&task_id).await {
            Ok(result) => {
                task_result = Some(result);
                break;
            }
            Err(QrmiError::TaskNotReady { .. }) => {
                sleep(Duration::from_millis(500)).await;
            }
            Err(error) => {
                panic!("task result retrieval failed: {error}");
            }
        }
    }

    let task_result = task_result.expect("task result should become available");

    let result_json: serde_json::Value =
        serde_json::from_str(&task_result.value).expect("task result should contain valid JSON");

    assert_eq!(result_json["type"], "test");
    assert_eq!(result_json["task_id"], task_id);
    assert!(result_json["parameters"].is_object());
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn task_status_reaches_completed_with_running_backend() {
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

    let mut completed = false;

    for _ in 0..30 {
        match qrmi
            .task_status(&task_id)
            .await
            .expect("task status retrieval should succeed")
        {
            TaskStatus::Queued | TaskStatus::Running => {
                sleep(Duration::from_millis(500)).await;
            }
            TaskStatus::Completed => {
                completed = true;
                break;
            }
            TaskStatus::Failed => {
                panic!("task should not fail");
            }
            TaskStatus::Cancelled => {
                panic!("task should not be cancelled");
            }
        }
    }

    assert!(completed, "task should reach Completed status");
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn task_stop_cancels_queued_task_with_running_backend() {
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

    qrmi.task_stop(&task_id)
        .await
        .expect("queued task cancellation should succeed");

    let status = qrmi
        .task_status(&task_id)
        .await
        .expect("cancelled task status retrieval should succeed");

    assert_eq!(status, TaskStatus::Cancelled);
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn target_returns_qpu_observables_with_running_backend() {
    let mut qrmi = LCCFQServer::new("lccfq_qpu").expect("LCCFQServer should be created");

    let target = qrmi
        .target()
        .await
        .expect("LCCFQ target retrieval should succeed");

    let target_json: serde_json::Value =
        serde_json::from_str(&target.value).expect("target should contain valid JSON");

    assert!(target_json["qubits"].is_object());
    assert!(target_json["last_updated"].is_string());
    assert!(!target_json["qubits"]
        .as_object()
        .expect("qubits should be a JSON object")
        .is_empty());
}

#[tokio::test]
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn task_logs_returns_lifecycle_logs_with_running_backend() {
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

    let mut completed = false;

    for _ in 0..30 {
        match qrmi
            .task_status(&task_id)
            .await
            .expect("task status retrieval should succeed")
        {
            TaskStatus::Queued | TaskStatus::Running => {
                sleep(Duration::from_millis(500)).await;
            }
            TaskStatus::Completed => {
                completed = true;
                break;
            }
            TaskStatus::Failed => {
                panic!("task should not fail");
            }
            TaskStatus::Cancelled => {
                panic!("task should not be cancelled");
            }
        }
    }

    assert!(completed, "task should reach Completed status");

    let logs = qrmi
        .task_logs(&task_id)
        .await
        .expect("task log retrieval should succeed");

    assert!(logs.contains("Task status changed to queued"));
    assert!(logs.contains("Task status changed to running"));
    assert!(logs.contains("Task status changed to completed"));
}
