use crate::cli::ServerConfig;
use crate::replication::{DbStateMachine, DbTypeConfig};
use ahnlich_replication::config::{ClusterLifecycle, RaftConfig, RaftStorageEngine};
use ahnlich_replication::network::GrpcRaftNetworkFactory;
use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::{
    AddLearnerRequest, ChangeMembershipRequest, GetLeaderRequest, GetMetricsRequest, NodeInfo,
};
use ahnlich_replication::storage::{ReplicationFailureState, StateMachineStore};
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use openraft::{Config as OpenRaftConfig, Membership, Raft, SnapshotPolicy, StoredMembership};
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use tonic::transport::{Channel, Endpoint};
use utils::server::ListenerStreamOrAddress;

const SERVICE_NAME: &str = "ahnlich-db";

pub(crate) type DbRaft = Raft<DbTypeConfig>;

pub(crate) struct ClusterRuntime {
    pub(crate) node_id: u64,
    pub(crate) raft_addr: SocketAddr,
    pub(crate) raft: Arc<DbRaft>,
    pub(crate) state_machine: StateMachineStore<DbTypeConfig, DbStateMachine>,
    pub(crate) failure_state: Arc<ReplicationFailureState>,
    cluster_listener: Mutex<Option<ListenerStreamOrAddress>>,
    leader_clients: Mutex<std::collections::HashMap<String, DbServiceClient<Channel>>>,
}

impl std::fmt::Debug for ClusterRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClusterRuntime")
            .field("node_id", &self.node_id)
            .field("raft_addr", &self.raft_addr)
            .field("failure_state", &self.failure_state)
            .finish_non_exhaustive()
    }
}

impl ClusterRuntime {
    fn new(
        node_id: u64,
        raft_addr: SocketAddr,
        raft: Arc<DbRaft>,
        state_machine: StateMachineStore<DbTypeConfig, DbStateMachine>,
        failure_state: Arc<ReplicationFailureState>,
        cluster_listener: ListenerStreamOrAddress,
    ) -> Self {
        Self {
            node_id,
            raft_addr,
            raft,
            state_machine,
            failure_state,
            cluster_listener: Mutex::new(Some(cluster_listener)),
            leader_clients: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub(crate) fn take_cluster_listener(&self) -> IoResult<ListenerStreamOrAddress> {
        self.cluster_listener
            .lock()
            .expect("cluster listener mutex poisoned")
            .take()
            .ok_or_else(|| std::io::Error::other("cluster listener already taken"))
    }

    pub(crate) fn leader_client(
        &self,
        service_addr: &str,
    ) -> Result<DbServiceClient<Channel>, String> {
        let mut clients = self
            .leader_clients
            .lock()
            .expect("leader client cache mutex poisoned");

        if let Some(client) = clients.get(service_addr) {
            return Ok(client.clone());
        }

        let endpoint = Endpoint::from_shared(format!("http://{service_addr}"))
            .map_err(|err| format!("invalid leader address {service_addr}: {err}"))?;
        let client = DbServiceClient::new(endpoint.connect_lazy());
        clients.insert(service_addr.to_owned(), client.clone());
        Ok(client)
    }

    fn node_info(&self, service_addr: SocketAddr) -> NodeInfo {
        NodeInfo {
            id: self.node_id,
            raft_addr: self.raft_addr.to_string(),
            service_addr: service_addr.to_string(),
        }
    }
}

fn hash_node_id(addr: SocketAddr) -> u64 {
    let mut hasher = DefaultHasher::new();
    addr.hash(&mut hasher);
    hasher.finish()
}

fn normalize_cluster_target(addr: SocketAddr) -> String {
    format!("http://{addr}")
}

fn build_raft_config(
    config: &ServerConfig,
    service_addr: SocketAddr,
    raft_addr: SocketAddr,
) -> RaftConfig {
    RaftConfig {
        node_id: hash_node_id(raft_addr),
        raft_addr,
        service_addr,
        storage: config.cluster_storage,
        data_dir: config.cluster_data_dir.clone(),
        snapshot_logs: config.cluster_snapshot_logs,
        snapshot_interval_ms: config.cluster_snapshot_interval,
        lifecycle: if config.cluster_bootstrap {
            ClusterLifecycle::Bootstrap
        } else if let Some(join) = config.cluster_join {
            ClusterLifecycle::Join(join)
        } else {
            ClusterLifecycle::Existing
        },
    }
}

async fn resolve_join_target(join_addr: SocketAddr) -> SocketAddr {
    let target = normalize_cluster_target(join_addr);
    let mut client = match ClusterAdminServiceClient::connect(target.clone()).await {
        Ok(client) => client,
        Err(err) => {
            log::warn!("Failed to connect to join target {join_addr}: {err}");
            return join_addr;
        }
    };

    match client
        .get_leader(tonic::Request::new(GetLeaderRequest {}))
        .await
    {
        Ok(response) => response
            .into_inner()
            .leader_addr
            .parse()
            .unwrap_or(join_addr),
        Err(_) => join_addr,
    }
}

pub(crate) async fn initialize_cluster_runtime(
    config: &ServerConfig,
    service_addr: SocketAddr,
    cluster: &ClusterRuntime,
) -> IoResult<()> {
    match build_raft_config(config, service_addr, cluster.raft_addr).lifecycle {
        ClusterLifecycle::Bootstrap => {
            let node = ReplicationNode {
                raft_addr: cluster.raft_addr.to_string(),
                service_addr: service_addr.to_string(),
            };
            cluster
                .raft
                .initialize(BTreeMap::from([(cluster.node_id, node)]))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
        }
        ClusterLifecycle::Join(join_addr) => {
            let target = resolve_join_target(join_addr).await;
            let mut client = ClusterAdminServiceClient::connect(normalize_cluster_target(target))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            let node = cluster.node_info(service_addr);

            client
                .add_learner(tonic::Request::new(AddLearnerRequest {
                    node: Some(node.clone()),
                }))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;

            let metrics = client
                .get_metrics(tonic::Request::new(GetMetricsRequest {}))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?
                .into_inner();

            let mut node_ids = metrics.voter_ids;
            if !node_ids.contains(&node.id) {
                node_ids.push(node.id);
            }
            client
                .change_membership(tonic::Request::new(ChangeMembershipRequest { node_ids }))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
        }
        ClusterLifecycle::Existing => {}
    }

    Ok(())
}

pub(crate) async fn build_cluster_runtime(
    config: &ServerConfig,
    service_addr: SocketAddr,
    cluster_listener: ListenerStreamOrAddress,
) -> IoResult<ClusterRuntime> {
    let raft_addr = cluster_listener.local_addr()?;
    let raft_config = build_raft_config(config, service_addr, raft_addr);
    let failure_state = Arc::new(ReplicationFailureState::default());
    let state_machine = StateMachineStore::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false))),
        StoredMembership::new(None, Membership::new(vec![], BTreeMap::new())),
        failure_state.clone(),
    );

    let openraft_config = OpenRaftConfig {
        cluster_name: SERVICE_NAME.to_owned(),
        snapshot_policy: SnapshotPolicy::LogsSinceLast(raft_config.snapshot_logs),
        ..Default::default()
    }
    .validate()
    .map_err(|err| std::io::Error::other(err.to_string()))?;

    let raft = match raft_config.storage {
        RaftStorageEngine::RocksDb => {
            let data_dir = raft_config.data_dir.ok_or_else(|| {
                std::io::Error::other("cluster_data_dir is required when cluster_storage=rocksdb")
            })?;
            let log_store = ahnlich_replication::storage::RocksLogStore::<DbTypeConfig>::open(
                data_dir.join("raft"),
            )
            .map_err(|err| std::io::Error::other(err.to_string()))?;
            Raft::new(
                raft_config.node_id,
                Arc::new(openraft_config),
                GrpcRaftNetworkFactory::<DbTypeConfig>::default(),
                log_store,
                state_machine.clone(),
            )
            .await
            .map_err(|err| std::io::Error::other(err.to_string()))?
        }
        RaftStorageEngine::Memory => {
            return Err(std::io::Error::other(
                "cluster_storage=memory is not supported by the DB server runtime",
            ));
        }
    };

    Ok(ClusterRuntime::new(
        raft_config.node_id,
        raft_addr,
        Arc::new(raft),
        state_machine,
        failure_state,
        cluster_listener,
    ))
}
