package client

import (
	"bytes"
	"encoding/binary"
	"net"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

const (
	// bufferSize        = 1024
	versionLength     = 5
	headerLength      = 8
	initialByteLength = 8
)

var header = []byte("AHNLICH;")

type connectionInfo struct {
	remoteAddr    string
	localAddr     string
	remoteNetwork string
	localNetwork  string
}

func (ah *connectionInfo) update(conn net.Conn) {
	ah.remoteAddr = conn.RemoteAddr().String()
	ah.localAddr = conn.LocalAddr().String()
	ah.remoteNetwork = conn.RemoteAddr().Network()
	ah.localNetwork = conn.LocalAddr().Network()
}

type ahnlichProtocol struct {
	connManager   *transport.ConnectionManager
	version       dbResponse.Version
	clientVersion dbResponse.Version
	*connectionInfo
}

// NewAhnlichProtocol creates a new ahnlichProtocol
func newAhnlichProtocol(cm *transport.ConnectionManager) (*ahnlichProtocol, error) {
	versions, err := ahnlichclientgo.GetVersions()
	if err != nil {
		return nil, err
	}

	return &ahnlichProtocol{
		connManager: cm,
		version: dbResponse.Version{
			Major: versions.Protocol.Major,
			Minor: versions.Protocol.Minor,
			Patch: versions.Protocol.Patch,
		},
		connectionInfo: &connectionInfo{},
		clientVersion: dbResponse.Version{
			Major: versions.Client.Major,
			Minor: versions.Client.Minor,
			Patch: versions.Client.Patch,
		},
	}, nil
}

// serializeQuery serializes the query request to be sent to the server
func (ap *ahnlichProtocol) serializeQuery(serverQuery *dbQuery.ServerQuery) ([]byte, error) {
	serverQueryBinCode, err := serverQuery.BincodeSerialize()
	if err != nil {
		return nil, err
	}
	versionBinCode, err := ap.version.BincodeSerialize()
	if err != nil {
		return nil, err
	}

	// Calculate the length of the serverQuery and convert it to an 8-byte little-endian format
	serverQueryLength := len(serverQueryBinCode)
	serverQueryLengthBytes := make([]byte, initialByteLength)
	binary.LittleEndian.PutUint64(serverQueryLengthBytes, uint64(serverQueryLength))

	// Concatenate header bytes, version bincode, serverQueryLengthBytes, and serverQuery bincode
	var buffer bytes.Buffer
	buffer.Write(header)
	buffer.Write(versionBinCode)
	buffer.Write(serverQueryLengthBytes)
	buffer.Write(serverQueryBinCode)

	data := buffer.Bytes()

	return data, nil
}

// deserializeResponse deserializes the serverQuery from the server
func (ap *ahnlichProtocol) deserializeResponse(data []byte) (*dbResponse.ServerResult, error) {
	result, err := dbResponse.BincodeDeserializeServerResult(data)
	if err != nil {
		return nil, err
	}
	return &result, nil
}

// send sends data to the ahnlich server using the protocol
func (ap *ahnlichProtocol) send(conn net.Conn, serverQuery *dbQuery.ServerQuery) error {
	data, err := ap.serializeQuery(serverQuery)
	if err != nil {
		return err
	}
	n, err := conn.Write(data)
	if err != nil {
		return err
	}
	if n != len(data) {
		return &utils.AhnlichClientException{Message: "Invalid Length"} // TODO: Convert to a protocol error
	}
	return nil
}

// Request sends data to the ahnlich server and receives a response using the protocol (Unary)
func (ap *ahnlichProtocol) request(serverQuery *dbQuery.ServerQuery) (*dbResponse.ServerResult, error) {
	conn, err := ap.connManager.GetConnection()
	ap.connectionInfo.update(conn)
	if err != nil {
		// TODO: Ask: Should we close the connection here or just return the error?
		// TODO: Implement a retry mechanism here or Refresh the connection pool
		return nil, err
	}
	defer ap.connManager.Return(conn)
	err = ap.send(conn, serverQuery)
	if err != nil {
		return nil, err
	}
	response, err := ap.receive(conn)
	if err != nil {
		return nil, err
	}
	return response, nil
}

// receive receives data from the ahnlich server using the protocol
func (ap *ahnlichProtocol) receive(conn net.Conn) (*dbResponse.ServerResult, error) {
	// Read the header
	headerBuffer := make([]byte, headerLength)
	n, err := conn.Read(headerBuffer)
	if err != nil {
		ap.connManager.Release() // TODO: Check if this is the right place to release the connection or just a close is enough. ALso is a panic better here?
		// TODO: Implement a retry mechanism here or Refresh the connection pool
		return nil, err
	}
	if !bytes.Equal(headerBuffer[:n], []byte(header)) {
		return nil, &utils.AhnlichClientException{Message: "Invalid header"} // TODO: Convert to a protocol error
	}

	// Read the version: Ignore the version for now
	_, err = conn.Read(make([]byte, versionLength))
	if err != nil {
		ap.connManager.Release()
		return nil, err
	}
	// Read the length of the expected response data
	lengthBuffer := make([]byte, initialByteLength)
	n, err = conn.Read(lengthBuffer)
	if err != nil {
		ap.connManager.Release()
		return nil, err
	}
	lengthToRead := binary.LittleEndian.Uint64(lengthBuffer[:n])
	// Read the data
	data := make([]byte, lengthToRead)
	n, err = conn.Read(data)
	if err != nil {
		ap.connManager.Release()
		return nil, err
	}
	if uint64(n) != lengthToRead {
		return nil, &utils.AhnlichClientException{Message: "Invalid Length"} // TODO: Convert to a protocol error
	}
	// Deserialize the response
	response, err := ap.deserializeResponse(data)
	if err != nil {
		return nil, err
	}
	return response, nil
}

// Close closes the connection to the server
func (ap *ahnlichProtocol) close() {
	ap.connManager.Release()
}
