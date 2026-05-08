//! Surface-level smoke tests
//!
//! Goals:
//! * Prove the crate compiles, links, and exposes the expected public API.
//! * Round-trip the application command/response types via serde_json (the
//!   same encoding the state machine and network transport use end-to-end).

use ahnlich_replication::types::{
    AiResponse, ClientWriteRequest, ClientWriteResponse, DbCommand, DbResponse,
};

#[test]
fn db_command_round_trip_via_json() {
    let cmd = DbCommand::Set(b"hello-world".to_vec());
    let req = ClientWriteRequest {
        command: cmd.clone(),
    };

    let encoded = serde_json::to_vec(&req).expect("encode");
    let decoded: ClientWriteRequest<DbCommand> = serde_json::from_slice(&encoded).expect("decode");

    match decoded.command {
        DbCommand::Set(payload) => assert_eq!(payload, b"hello-world"),
        other => panic!("unexpected command variant: {other:?}"),
    }
}

#[test]
fn response_envelope_round_trips() {
    let db = DbResponse::Bytes(vec![1, 2, 3]);
    let envelope = ClientWriteResponse { response: db };
    let encoded = serde_json::to_vec(&envelope).expect("encode");
    let decoded: ClientWriteResponse<DbResponse> =
        serde_json::from_slice(&encoded).expect("decode");
    match decoded.response {
        DbResponse::Bytes(payload) => assert_eq!(payload, vec![1, 2, 3]),
        DbResponse::Unit => panic!("expected Bytes variant"),
    }

    let ai = AiResponse::Unit;
    let encoded = serde_json::to_vec(&ai).expect("encode");
    let decoded: AiResponse = serde_json::from_slice(&encoded).expect("decode");
    assert!(matches!(decoded, AiResponse::Unit));
}
