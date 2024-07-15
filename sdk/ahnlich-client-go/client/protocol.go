package client

import (
	"bytes"
	"encoding/binary"
	"net"
	"time"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// AhnlichProtocol handles the custom communication protocol
type AhnlichProtocol struct {
	connManager   *transport.ConnectionManager
	version       dbResponse.Version
	cfg           ahnlichclientgo.ProtocolConfig
	clientVersion dbResponse.Version
}

// NewAhnlichProtocol creates a new AhnlichProtocol
func NewAhnlichProtocol(cm *transport.ConnectionManager, cfg ahnlichclientgo.ProtocolConfig) (*AhnlichProtocol, error) {
	versions, err := ahnlichclientgo.GetVersions()
	if err != nil {
		return nil, err
	}

	return &AhnlichProtocol{
		connManager: cm,
		version: dbResponse.Version{
			Major: versions.Protocol.Major,
			Minor: versions.Protocol.Minor,
			Patch: versions.Protocol.Patch,
		},
		cfg: cfg,
		clientVersion: dbResponse.Version{
			Major: versions.Client.Major,
			Minor: versions.Client.Minor,
			Patch: versions.Client.Patch,
		},
	}, nil
}

// serializeQuery serializes the query request to be sent to the server
func (ap *AhnlichProtocol) serializeQuery(serverQuery *dbQuery.ServerQuery) ([]byte, error) {
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
	serverQueryLengthBytes := make([]byte, ap.cfg.DefaultLength)
	binary.LittleEndian.PutUint64(serverQueryLengthBytes, uint64(serverQueryLength))

	// Concatenate Header bytes, version bincode, serverQueryLengthBytes, and serverQuery bincode
	var buffer bytes.Buffer
	buffer.Write(ap.cfg.Header)
	buffer.Write(versionBinCode)
	buffer.Write(serverQueryLengthBytes)
	buffer.Write(serverQueryBinCode)

	data := buffer.Bytes()

	return data, nil
}

// deserializeResponse deserializes the serverQuery from the server
func (ap *AhnlichProtocol) deserializeResponse(data []byte) (*dbResponse.ServerResult, error) {
	result, err := dbResponse.BincodeDeserializeServerResult(data)
	if err != nil {
		return nil, err
	}
	return &result, nil
}

// send sends data to the ahnlich server using the protocol
func (ap *AhnlichProtocol) send(conn net.Conn, serverQuery *dbQuery.ServerQuery) error {
	err := conn.SetWriteDeadline(time.Now().Add(ap.connManager.Cfg.WriteTimeout))
	if err != nil {
		return err
	}
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
func (ap *AhnlichProtocol) request(serverQuery *dbQuery.ServerQuery) (*dbResponse.ServerResult, error) {
	conn, err := ap.connManager.GetConnection()
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
func (ap *AhnlichProtocol) receive(conn net.Conn) (*dbResponse.ServerResult, error) {
	// Set timeout for reading data from the server
	err := conn.SetReadDeadline(time.Now().Add(ap.connManager.Cfg.ReadTimeout))
	if err != nil {
		return nil, err
	}
	// Read the header
	headerBuffer := make([]byte, ap.cfg.HeaderLength)
	n, err := conn.Read(headerBuffer)
	if err != nil {
		ap.connManager.Release() // TODO: Check if this is the right place to release the connection or just a close is enough. ALso is a panic better here?
		// TODO: Implement a retry mechanism here or Refresh the connection pool
		return nil, err
	}
	if !bytes.Equal(headerBuffer[:n], []byte(ap.cfg.Header)) {
		return nil, &utils.AhnlichClientException{Message: "Invalid Header"} // TODO: Convert to a protocol error
	}

	// Read the version: Ignore the version for now
	_, err = conn.Read(make([]byte, ap.cfg.VersionLength))
	if err != nil {
		ap.connManager.Release()
		return nil, err
	}
	// Read the length of the expected response data
	lengthBuffer := make([]byte, ap.cfg.DefaultLength)
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
func (ap *AhnlichProtocol) close() {
	ap.connManager.Release()
}
