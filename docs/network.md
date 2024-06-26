# Network Thoughts

- Communication happens over TCP. Can look into using tokio::net::TcpListener and tokio_util::codec::Framed;
- Clients must be able to send their versions within a connect message over TCP and the server can accept/reject the version depending on whether or not the client major version number differs over the server major version number.
- Clients must be able to send commands to disconnect their session to the server and get disconnected. They can also send a shutdown message and have the server perform cleanup which would entail, stopping responding to new requests, storing stores within persistence if persistence is available and then exist gracefully.
- Server must be configurable using `clap` module to setup things like host, port, persistence capabilites (persistence means the ability to sync to hard disk and retrieve upon start). There are other specific network capabilites that can be configurable such as authentication with username and password or even encryption but those are for later down the line.
- Server must be able to serve in nonblocking mode i.e multiple requests to the same/different stores can be responded to with responses from the server using futures/threads
- Server errors must also be Serializable, Deserializable. This is because they have to also be sent across the wire incase the client makes a request that results in a server error
- Server must be able to give a report of all network clients connected at any given time.
- Messages can be sent across the wire using BinCode/MessagePack as both are supported by serde and all datastructuresused in the messages e.g ndarray, are serde compatible.
- Clients must be able to connect to server and then subsequently use to issue every possible Query. Subsequently we can look into using r2d2/deadpool to enable connection pooling to hold on to a pool of connections to the ahnlich server. We can also look into sending a batch of commands all at once (pipelining) rather than one at a time.
