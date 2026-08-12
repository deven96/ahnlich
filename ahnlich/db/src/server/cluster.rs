use crate::cli::ServerConfig;
use crate::replication::{DbStateMachine, DbTypeConfig};
use ahnlich_replication::config::{ClusterLifecycle, RaftConfig, RaftStorageEngine};
use ahnlich_replication::identity::{
    ClusterIdentity, REPLICATION_PROTOCOL_VERSION, STATE_MACHINE_FORMAT_VERSION,
};
use ahnlich_replication::network::GrpcRaftNetworkFactory;
use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::{
    AdmitLearnerRequest, CandidateNode, GetClusterIdentityRequest, GetLeaderRequest, NodeInfo,
    PromoteLearnerRequest,
};
use ahnlich_replication::storage::{ReplicationFailureState, RocksLogStore, StateMachineStore};
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use openraft::{Config as OpenRaftConfig, Membership, Raft, SnapshotPolicy, StoredMembership};
use rand::random;
use std::collections::{BTreeMap, HashMap};
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
    pub(crate) raft_bind_addr: SocketAddr,
    pub(crate) raft_advertise_addr: SocketAddr,
    pub(crate) raft_leader_forwarding_advertise_addr: SocketAddr,
    pub(crate) lifecycle: ClusterLifecycle,
    pub(crate) raft: Arc<DbRaft>,
    pub(crate) state_machine: StateMachineStore<DbTypeConfig, DbStateMachine>,
    pub(crate) failure_state: Arc<ReplicationFailureState>,
    pub(crate) identity_store: RocksLogStore<DbTypeConfig>,
    cluster_listener: Mutex<Option<ListenerStreamOrAddress>>,
    leader_clients: Mutex<HashMap<String, DbServiceClient<Channel>>>,
}

impl std::fmt::Debug for ClusterRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClusterRuntime")
            .field("node_id", &self.node_id)
            .field("raft_bind_addr", &self.raft_bind_addr)
            .field("raft_advertise_addr", &self.raft_advertise_addr)
            .field(
                "raft_leader_forwarding_advertise_addr",
                &self.raft_leader_forwarding_advertise_addr,
            )
            .field("failure_state", &self.failure_state)
            .finish_non_exhaustive()
    }
}

impl ClusterRuntime {
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

    fn node_info(&self) -> NodeInfo {
        NodeInfo {
            id: self.node_id,
            raft_addr: self.raft_advertise_addr.to_string(),
            service_addr: self.raft_leader_forwarding_advertise_addr.to_string(),
        }
    }
}

fn cluster_identity(config: &ServerConfig) -> ClusterIdentity {
    ClusterIdentity::new(
        format!("{:032x}", random::<u128>()),
        config.cluster_name.clone(),
    )
}

fn persist_bootstrap_cluster_identity(
    config: &ServerConfig,
    cluster: &ClusterRuntime,
) -> IoResult<()> {
    if cluster
        .identity_store
        .cluster_identity()
        .map_err(|err| std::io::Error::other(err.to_string()))?
        .is_none()
    {
        cluster
            .identity_store
            .persist_cluster_identity(&cluster_identity(config))
            .map_err(|err| std::io::Error::other(err.to_string()))?;
    }

    Ok(())
}

fn persist_joined_cluster_identity(
    cluster: &ClusterRuntime,
    identity: ClusterIdentity,
) -> IoResult<()> {
    cluster
        .identity_store
        .persist_cluster_identity(&identity)
        .map_err(|err| std::io::Error::other(err.to_string()))
}

fn normalize_cluster_target(addr: SocketAddr) -> String {
    format!("http://{addr}")
}

fn build_raft_config(
    config: &ServerConfig,
    node_id: u64,
    service_addr: SocketAddr,
    raft_addr: SocketAddr,
    lifecycle: ClusterLifecycle,
) -> RaftConfig {
    RaftConfig {
        node_id,
        raft_addr,
        service_addr,
        storage: config.cluster_storage,
        data_dir: config.cluster_data_dir.clone(),
        snapshot_logs: config.cluster_snapshot_logs,
        snapshot_interval_ms: config.cluster_snapshot_interval,
        lifecycle,
    }
}

fn cluster_lifecycle(config: &ServerConfig) -> ClusterLifecycle {
    if config.cluster_bootstrap {
        ClusterLifecycle::Bootstrap
    } else if let Some(join_addr) = config.cluster_join {
        ClusterLifecycle::Join(join_addr)
    } else {
        ClusterLifecycle::Existing
    }
}

fn advertised_addr(
    configured: Option<SocketAddr>,
    bound: SocketAddr,
    flag: &str,
) -> IoResult<SocketAddr> {
    match configured {
        Some(addr) if addr.ip().is_unspecified() => Err(std::io::Error::other(format!(
            "{flag} must not use an unspecified address",
        ))),
        Some(addr) => Ok(addr),
        None if bound.ip().is_unspecified() => Err(std::io::Error::other(format!(
            "{flag} is required when the listener binds to {bound}",
        ))),
        None => Ok(bound),
    }
}

fn generate_node_id() -> u64 {
    loop {
        let node_id = random::<u64>();
        if node_id != 0 {
            return node_id;
        }
    }
}

fn load_or_create_node_id(
    store: &RocksLogStore<DbTypeConfig>,
    lifecycle: &ClusterLifecycle,
) -> IoResult<u64> {
    match store
        .node_id()
        .map_err(|err| std::io::Error::other(err.to_string()))?
    {
        Some(node_id) => Ok(node_id),
        None if matches!(lifecycle, ClusterLifecycle::Existing) => Err(std::io::Error::other(
            "node id is missing from existing Raft storage",
        )),
        None => {
            let node_id = generate_node_id();
            store
                .persist_node_id(node_id)
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            Ok(node_id)
        }
    }
}

fn cluster_identity_from_response(
    response: ahnlich_replication::proto::cluster_admin::GetClusterIdentityResponse,
) -> ClusterIdentity {
    ClusterIdentity {
        id: response.cluster_id,
        name: response.cluster_name,
        replication_protocol_version: response.replication_protocol_version,
        state_machine_format_version: response.state_machine_format_version,
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
    cluster: &ClusterRuntime,
) -> IoResult<()> {
    match &cluster.lifecycle {
        ClusterLifecycle::Bootstrap => {
            persist_bootstrap_cluster_identity(config, cluster)?;
            let node = ReplicationNode {
                raft_addr: cluster.raft_advertise_addr.to_string(),
                service_addr: cluster.raft_leader_forwarding_advertise_addr.to_string(),
            };
            cluster
                .raft
                .initialize(BTreeMap::from([(cluster.node_id, node)]))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
        }
        ClusterLifecycle::Join(join_addr) => {
            let target = resolve_join_target(*join_addr).await;
            let mut client = ClusterAdminServiceClient::connect(normalize_cluster_target(target))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            let node = cluster.node_info();

            let identity = client
                .get_cluster_identity(tonic::Request::new(GetClusterIdentityRequest {}))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?
                .into_inner();
            let identity = cluster_identity_from_response(identity);
            identity
                .ensure_supported_by_current_binary()
                .map_err(std::io::Error::other)?;
            persist_joined_cluster_identity(cluster, identity)?;

            client
                .admit_learner(tonic::Request::new(AdmitLearnerRequest {
                    candidate: Some(CandidateNode {
                        node: Some(node),
                        replication_protocol_version: REPLICATION_PROTOCOL_VERSION,
                        state_machine_format_version: STATE_MACHINE_FORMAT_VERSION,
                    }),
                }))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;

            client
                .promote_learner(tonic::Request::new(PromoteLearnerRequest {
                    node_id: cluster.node_id,
                }))
                .await
                .map_err(|err| std::io::Error::other(err.to_string()))?;
        }
        ClusterLifecycle::Existing => {
            let identity = cluster
                .identity_store
                .cluster_identity()
                .map_err(|err| std::io::Error::other(err.to_string()))?
                .ok_or_else(|| {
                    std::io::Error::other("cluster identity is missing from existing Raft storage")
                })?;
            identity
                .ensure_supported_by_current_binary()
                .map_err(std::io::Error::other)?;
        }
    }

    Ok(())
}

pub(crate) async fn build_cluster_runtime(
    config: &ServerConfig,
    service_addr: SocketAddr,
    cluster_listener: ListenerStreamOrAddress,
) -> IoResult<ClusterRuntime> {
    let raft_bind_addr = cluster_listener.local_addr()?;
    let raft_advertise_addr = advertised_addr(
        config.cluster_advertise_addr,
        raft_bind_addr,
        "--cluster-advertise-addr",
    )?;
    let raft_leader_forwarding_advertise_addr = advertised_addr(
        config.cluster_leader_forwarding_advertise_addr,
        service_addr,
        "--cluster-leader-forwarding-advertise-addr",
    )?;
    let lifecycle = cluster_lifecycle(config);
    let failure_state = Arc::new(ReplicationFailureState::default());

    if !matches!(config.cluster_storage, RaftStorageEngine::RocksDb) {
        return Err(std::io::Error::other(
            "cluster_storage=memory is not supported by the DB server runtime",
        ));
    }

    let data_dir = config.cluster_data_dir.clone().ok_or_else(|| {
        std::io::Error::other("cluster_data_dir is required when cluster_storage=rocksdb")
    })?;
    let log_store = RocksLogStore::<DbTypeConfig>::open(data_dir.join("raft"))
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let node_id = load_or_create_node_id(&log_store, &lifecycle)?;
    let raft_config = build_raft_config(
        config,
        node_id,
        raft_leader_forwarding_advertise_addr,
        raft_advertise_addr,
        lifecycle.clone(),
    );

    let openraft_config = OpenRaftConfig {
        cluster_name: SERVICE_NAME.to_owned(),
        snapshot_policy: SnapshotPolicy::LogsSinceLast(raft_config.snapshot_logs),
        ..Default::default()
    }
    .validate()
    .map_err(|err| std::io::Error::other(err.to_string()))?;

    let state_machine = StateMachineStore::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false))),
        StoredMembership::new(None, Membership::new(vec![], BTreeMap::new())),
        failure_state.clone(),
        Arc::new(log_store.clone()),
    )
    .map_err(|err| std::io::Error::other(err.to_string()))?;
    let raft = Raft::new(
        raft_config.node_id,
        Arc::new(openraft_config),
        GrpcRaftNetworkFactory::<DbTypeConfig>::default(),
        log_store.clone(),
        state_machine.clone(),
    )
    .await
    .map_err(|err| std::io::Error::other(err.to_string()))?;

    Ok(ClusterRuntime {
        node_id: raft_config.node_id,
        raft_bind_addr,
        raft_advertise_addr,
        raft_leader_forwarding_advertise_addr,
        lifecycle,
        raft: Arc::new(raft),
        state_machine,
        failure_state,
        identity_store: log_store,
        cluster_listener: Mutex::new(Some(cluster_listener)),
        leader_clients: Mutex::new(HashMap::new()),
    })
}
