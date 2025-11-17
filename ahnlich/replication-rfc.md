# Ahnlich replication RFC

The first thing we want to do is to identify the components we need to implement.

They are (NOTE: these names are tentative):

- `LogStore` - This is where the logs from the Raft cluster activities will be stored. Here is an in-memory impl from the openraft guys that they used in their example: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/mem-log/src/log_store.rs>, and I believe we can easily co-opt this into a write-to-disk-log file service, or any other log storage we settle on.

- `StateMachineStore` - This is where the last known state (snapshot) is stored and read from. They have a neat impl here as well: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/store/mod.rs>, and I think the bit we need to figure out are where we want to 'store' the state machine.

- `Network` - This is the communication layer for the nodes. They have a tonic gRPC impl here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/network/mod.rs>, we can adopt from.

- `Raft` - This is the openraft-rs algorithm impl. Here's were just simply importing from their package and passing in our stores and network, as een here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/app.rs#L35>.

- `RaftService` - This is the Raft consensus gRPC service. Basically the voting, log replication and snapshot application, as you can see here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/grpc/raft_service.rs>.

- `AppService` - This is the client/application gRPC service. Here, the agent/app running the Raft cluster can issue commands to change the state, behaviour and roles of the nodes in the cluster. Impl here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/grpc/app_service.rs>.

- `Server` - This is the server to add the above services to, and essentially listen for the requests. It's a `tonic` Server, so we're just importing from tonic, adding our services and giving it a port to listen to requests at, like they do in their example here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/app.rs#L47>

For the App and Raft Service, they have a bunch of types defined in protobuf, as seen here: <https://github.com/databendlabs/openraft/tree/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/proto>, and they seem to be importing them directly (without a pre-generation step) into their Rust code using tonic, as shown here: <https://github.com/databendlabs/openraft/blob/4f0fd5fa034413d2f367306da4a0016f7603fb7e/examples/raft-kv-memstore-grpc/src/lib.rs#L8>

`AhnlichRaftService` - this is where were are going to plug in Ahnlich, by adding some service that will instantiate the Faft service based on some config/commands passed by the CLI. I'm not sure exactly what goes here yet, but i think some of the things we would want to do here are:

- Allow for Raft nodes to be created based on some config/command issued via the CLI

- Pass some Ahnlich data to the Raft cluster that it would replicate through out the cluster

- Allow for the cluster to be started and stopped

- Allow for the cluster to be restarted

The next step I think is answering the following questions.

1. Where/how are we storing the logs?

1. Where/how are we storing the state machine snapshots?

1. What/what else goes in `AhnlichRaftService`?

Then we can chose one of the above components to start implementing, to help us gain clarity.
