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
use crate::models::ResourceType;
use crate::QuantumResource;

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
#[ignore = "requires a running LCCFQ Backend with valid certificates"]
async fn is_accessible_with_running_backend() {
    let mut qrmi = LCCFQServer::new("lccfq_qpu").expect("LCCFQServer should be created");

    let accessible = qrmi
        .is_accessible()
        .await
        .expect("LCCFQ Ping should succeed");

    assert!(accessible);
}
