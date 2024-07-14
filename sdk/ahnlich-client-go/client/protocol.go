package client

import (
	"bytes"
	"encoding/binary"
	"time"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// AhnlichProtocol handles the custom communication protocol
type AhnlichProtocol struct {
    connManager *transport.ConnectionManager
	Version dbResponse.Version
	cfg ahnlichclientgo.Config
	ClientVersion dbResponse.Version
}

// NewAhnlichProtocol creates a new AhnlichProtocol
func NewAhnlichProtocol(cm *transport.ConnectionManager, cfg ahnlichclientgo.Config) (*AhnlichProtocol, error){
	versions,err := ahnlichclientgo.GetVersions()
	if err != nil {
		return nil, err
	}

    return &AhnlichProtocol{
		connManager: cm,
		Version: 	dbResponse.Version{
		Major: versions.Protocol.Major,
		Minor: versions.Protocol.Minor,
		Patch: versions.Protocol.Patch,
	},
		cfg : cfg,
		ClientVersion: dbResponse.Version{
			Major: versions.Client.Major,
			Minor: versions.Client.Minor,
			Patch: versions.Client.Patch,
		},
		},nil
}

// serializeQuery serializes the query request to be sent to the server
func (ap *AhnlichProtocol) serializeQuery(serverQuery *dbQuery.ServerQuery) ([]byte,error) {
    serverQueryBinCode,err := serverQuery.BincodeSerialize()
	if err != nil {
		return nil, err
	}
	versionBinCode,err := ap.Version.BincodeSerialize()
	if err != nil {
		return nil, err
	}
	// Get the length of the byte slice
	length := len(serverQueryBinCode)

	// Create a buffer to hold the 8-byte array
	buf := new(bytes.Buffer)

	// Write the length as an 8-byte little-endian value
	err = binary.Write(buf, binary.LittleEndian, int64(length))
	if err != nil {
		// fmt.Println("binary.Write failed:", err)
		return nil, err
	}
	// Get the byte array
	serverQueryLengthBytes := buf.Bytes()

	// Concatenate the byte arrays
	data := append(ap.cfg.Header,versionBinCode...)
	data = append(data, serverQueryLengthBytes...)
	data = append(data, serverQueryBinCode...)
	return data,nil
}

// deserializeResponse deserializes the response from the server
func (ap *AhnlichProtocol) deserializeResponse(data []byte) (*dbResponse.ServerResult,error) {
	result,err :=dbResponse.BincodeDeserializeServerResult(data)
	if err != nil {
		return nil, err
	}
	return &result,nil
}

// Send sends data to the ahnlich server using the protocol
func (ap *AhnlichProtocol) Send(serverQuery *dbQuery.ServerQuery) (error) {
	conn, err := ap.connManager.GetConnection()
	// Set timeout for writing data to the server
	conn.SetWriteDeadline(time.Now().Add(ap.cfg.WriteTimeout))
    if err != nil {
		// TODO: Ask: Should we close the connection here or just return the error?
		// TODO: Implement a retry mechanism here or Refresh the connection pool
        return err
    }
    defer ap.connManager.Return(conn)

	data,err := ap.serializeQuery(serverQuery)
	if err != nil {
		return err
	}
    _, err = conn.Write(data)
    if err != nil {
        return err
    }
	return nil
}

// SendReceive sends data to the ahnlich server and receives a response using the protocol (Unary)
func (ap *AhnlichProtocol) SendReceive(serverQuery *dbQuery.ServerQuery) (*dbResponse.ServerResult, error) {
	err := ap.Send(serverQuery)
	if err != nil {
		return nil, err
	}
	response,err := ap.Receive()
	if err != nil {
		return nil, err
	}
	return response,nil
}

// Receive receives data from the ahnlich server using the protocol
func (ap *AhnlichProtocol) Receive() (*dbResponse.ServerResult, error) {
	conn, err := ap.connManager.GetConnection()
	// Set timeout for reading data from the server
	conn.SetReadDeadline(time.Now().Add(ap.cfg.ReadTimeout))
    if err != nil {
        return nil,err
    }
    defer ap.connManager.Return(conn)

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

	// Read the length
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
	response,err := ap.deserializeResponse(data)
	if err != nil {
		return nil, err
	}
	return response,nil
}

// Close closes the connection to the server
func (ap *AhnlichProtocol) Close() {
	ap.connManager.Release()
}



