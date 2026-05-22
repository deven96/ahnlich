use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::io::Cursor;

use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::types::{DbCommand, DbResponse};
use bitcode::{deserialize as bitcode_deserialize, serialize as bitcode_serialize};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use openraft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use openraft::{
    CommittedLeaderId, Entry, EntryPayload, LogId, Membership, SnapshotMeta, StoredMembership, Vote,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

openraft::declare_raft_types!(
    pub BenchConfig:
        D = DbCommand,
        R = DbResponse,
        Node = ReplicationNode,
        SnapshotData = Cursor<Vec<u8>>
);

fn criterion_config() -> Criterion {
    Criterion::default().sample_size(20)
}

fn log_id(term: u64, node_id: u64, index: u64) -> LogId<u64> {
    LogId::new(CommittedLeaderId::new(term, node_id), index)
}

fn replication_node(id: u64) -> ReplicationNode {
    ReplicationNode {
        raft_addr: format!("10.0.0.{id}:19000"),
        service_addr: format!("10.0.0.{id}:18000"),
    }
}

fn membership(voters: &[u64]) -> Membership<u64, ReplicationNode> {
    let voter_set = voters.iter().copied().collect::<BTreeSet<_>>();
    let nodes = voters
        .iter()
        .copied()
        .map(|id| (id, replication_node(id)))
        .collect::<BTreeMap<_, _>>();
    Membership::new(vec![voter_set], nodes)
}

fn command_payload(bytes: usize) -> Vec<u8> {
    (0..bytes).map(|i| (i % 251) as u8).collect()
}

fn append_entries_request(
    entry_count: usize,
    payload_bytes: usize,
) -> AppendEntriesRequest<BenchConfig> {
    let entries = (0..entry_count)
        .map(|i| Entry {
            log_id: log_id(7, 1, (i + 1) as u64),
            payload: EntryPayload::Normal(DbCommand::Set(command_payload(payload_bytes))),
        })
        .collect();

    AppendEntriesRequest {
        vote: Vote::new_committed(7, 1),
        prev_log_id: Some(log_id(7, 1, 0)),
        entries,
        leader_commit: Some(log_id(7, 1, entry_count as u64)),
    }
}

fn append_entries_response() -> AppendEntriesResponse<u64> {
    AppendEntriesResponse::PartialSuccess(Some(log_id(7, 1, 32)))
}

fn vote_request() -> VoteRequest<u64> {
    VoteRequest::new(Vote::new(9, 3), Some(log_id(8, 2, 512)))
}

fn vote_response() -> VoteResponse<u64> {
    VoteResponse::new(Vote::new(9, 3), Some(log_id(8, 2, 512)), true)
}

fn install_snapshot_request(chunk_bytes: usize) -> InstallSnapshotRequest<BenchConfig> {
    InstallSnapshotRequest {
        vote: Vote::new_committed(11, 2),
        meta: SnapshotMeta {
            last_log_id: Some(log_id(11, 2, 4096)),
            last_membership: StoredMembership::new(
                Some(log_id(11, 2, 4090)),
                membership(&[1, 2, 3]),
            ),
            snapshot_id: "11-2-4096".to_string(),
        },
        offset: 0,
        data: command_payload(chunk_bytes),
        done: true,
    }
}

fn install_snapshot_response() -> InstallSnapshotResponse<u64> {
    InstallSnapshotResponse {
        vote: Vote::new_committed(11, 2),
    }
}

fn json_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("json encode")
}

fn bitcode_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    bitcode_serialize(value).expect("bitcode encode")
}

fn report_case_sizes<T: Serialize>(label: &str, value: &T) {
    let json = json_bytes(value);
    let bitcode = bitcode_bytes(value);
    let ratio = bitcode.len() as f64 / json.len() as f64;
    eprintln!(
        "[raft_rpc_codec] {label}: json={} bytes, bitcode={} bytes ({ratio:.2}x of json)",
        json.len(),
        bitcode.len(),
    );
}

fn bench_codec_round_trip<T>(c: &mut Criterion, group_name: &str, label: &str, value: &T)
where
    T: Serialize + DeserializeOwned + 'static,
{
    report_case_sizes(label, value);

    let json_payload = json_bytes(value);
    let bitcode_payload = bitcode_bytes(value);
    let throughput_bytes = json_payload.len().max(bitcode_payload.len()) as u64;

    let mut serialize_group = c.benchmark_group(format!("{group_name}_serialize"));
    serialize_group.throughput(Throughput::Bytes(throughput_bytes));
    serialize_group.bench_with_input(BenchmarkId::new("json", label), value, |b, value| {
        b.iter(|| black_box(serde_json::to_vec(black_box(value)).expect("json encode")));
    });
    serialize_group.bench_with_input(BenchmarkId::new("bitcode", label), value, |b, value| {
        b.iter(|| black_box(bitcode_serialize(black_box(value)).expect("bitcode encode")));
    });
    serialize_group.finish();

    let mut deserialize_group = c.benchmark_group(format!("{group_name}_deserialize"));
    deserialize_group.throughput(Throughput::Bytes(throughput_bytes));
    deserialize_group.bench_with_input(
        BenchmarkId::new("json", label),
        &json_payload,
        |b, payload| {
            b.iter(|| {
                black_box(serde_json::from_slice::<T>(black_box(payload)).expect("json decode"))
            });
        },
    );
    deserialize_group.bench_with_input(
        BenchmarkId::new("bitcode", label),
        &bitcode_payload,
        |b, payload| {
            b.iter(|| {
                black_box(bitcode_deserialize::<T>(black_box(payload)).expect("bitcode decode"))
            });
        },
    );
    deserialize_group.finish();
}

fn bench_append_entries(c: &mut Criterion) {
    for (entry_count, payload_bytes) in [(1, 256), (10, 1024), (100, 4096)] {
        let label = format!("{entry_count}_entries_{payload_bytes}_bytes");
        let request = append_entries_request(entry_count, payload_bytes);
        bench_codec_round_trip(c, "append_entries_request", &label, &request);
    }

    let response = append_entries_response();
    bench_codec_round_trip(c, "append_entries_response", "partial_success", &response);
}

fn bench_vote(c: &mut Criterion) {
    bench_codec_round_trip(c, "vote_request", "standard", &vote_request());
    bench_codec_round_trip(c, "vote_response", "granted", &vote_response());
}

fn bench_install_snapshot(c: &mut Criterion) {
    for chunk_bytes in [64 * 1024, 512 * 1024] {
        let label = format!("chunk_{chunk_bytes}_bytes");
        let request = install_snapshot_request(chunk_bytes);
        bench_codec_round_trip(c, "install_snapshot_request", &label, &request);
    }

    bench_codec_round_trip(
        c,
        "install_snapshot_response",
        "standard",
        &install_snapshot_response(),
    );
}

criterion_group! {
    name = raft_rpc_codec;
    config = criterion_config();
    targets = bench_append_entries, bench_vote, bench_install_snapshot
}
criterion_main!(raft_rpc_codec);
