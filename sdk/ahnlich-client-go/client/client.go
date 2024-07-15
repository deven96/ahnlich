package client

import (
	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
)

type AhnlichClient struct {
	protocol *AhnlichProtocol
	pipeline *AhnlichDBQueryBuilder
}

// NewAhnlichClient creates a new instance of AhnlichClient
func NewAhnlichClient(cm *transport.ConnectionManager, cfg ahnlichclientgo.ClientConfig) (*AhnlichClient, error) {
	protocol, err := NewAhnlichProtocol(cm, cfg.ProtocolConfig)
	if err != nil {
		return nil, err
	}
	return &AhnlichClient{
		protocol: protocol,
		pipeline: NewAhnlichDBQueryBuilder(),
	}, nil
}

// Request sends the queries in the pipeline to the ahnlich db server and returns the response
func (ac *AhnlichClient) Request() (*dbResponse.ServerResult, error) {
	serverQuery, err := ac.pipeline.ParseBuildQueryToServer()
	if err != nil {
		return nil, err
	}
	response, err := ac.protocol.request(serverQuery)
	if err != nil {
		return nil, err
	}
	return response, nil
}

// Close closes the connection to the server
func (ac *AhnlichClient) Close() {
	ac.protocol.close()
}

// ProtocolVersion returns the version of the server protocol
func (ac *AhnlichClient) ProtocolVersion() (dbResponse.Version, error) {
	return ac.protocol.version, nil
}

// Version returns the version of the client
func (ac *AhnlichClient) Version() (dbResponse.Version, error) {
	return ac.protocol.clientVersion, nil
}

func (ac *AhnlichClient) Ping() (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildPingQuery()
	return ac.Request()
}

// Pipeline returns the pipeline for the client
func (ac *AhnlichClient) Pipeline() *AhnlichDBQueryBuilder {
	return ac.pipeline
}

// SetPipeline sets the pipeline for the client
func (ac *AhnlichClient) SetPipeline(pipeline *AhnlichDBQueryBuilder) {
	ac.pipeline = pipeline
}

func (ac *AhnlichClient) ServerInfo() (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildInfoServerQuery()
	return ac.Request()
}

func (ac *AhnlichClient) ListClients() (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildListClientsQuery()
	return ac.Request()
}

func (ac *AhnlichClient) ListStores() (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildListStoresQuery()
	return ac.Request()
}

func (ac *AhnlichClient) CreatePredicateIndex(storeName string, predicates []string) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildCreatePredicateIndexQuery(storeName, predicates)
	return ac.Request()
}

// func (ac *AhnlichClient) CreateStore(storeName string, dimension uint64, predicates []string, nonLinearAlgorithm []dbResponse.NonLinearAlgorithm, errorIfExist bool) (*dbResponse.ServerResult, error) {
// 	ac.pipeline.BuildCreateStoreQuery(storeName, dimension, predicates, nonLinearAlgorithm, errorIfExist)
// 	return ac.Request()
// }

func (ac *AhnlichClient) Set(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildSetQuery(storeName, inputs)
	return ac.Request()
}

func (ac *AhnlichClient) GetByKeys(storeName string, keys []dbQuery.Array) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildGetByKeysQuery(storeName, keys)
	return ac.Request()
}

func (ac *AhnlichClient) GetByPredicate(storeName string, condition dbQuery.PredicateCondition) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildGetByPredicateQuery(storeName, condition)
	return ac.Request()
}

func (ac *AhnlichClient) GetBySimN(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildGetBySimNQuery(storeName, searchInput, closest_n, algorithm, condition)
	return ac.Request()
}

func (ac *AhnlichClient) DropPredicateIndex(storeName string, predicates []string, errorIfNotExist bool) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildDropPredicateIndexQuery(storeName, predicates, errorIfNotExist)
	return ac.Request()
}

func (ac *AhnlichClient) DeleteKeys(storeName string, keys []dbQuery.Array) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildDeleteKeysQuery(storeName, keys)
	return ac.Request()
}

func (ac *AhnlichClient) DeletePredicate(storeName string, condition dbQuery.PredicateCondition) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildDeletePredicateQuery(storeName, condition)
	return ac.Request()
}

func (ac *AhnlichClient) DropStore(storeName string, errorIfNotExist bool) (*dbResponse.ServerResult, error) {
	ac.pipeline.BuildDropStoreQuery(storeName, errorIfNotExist)
	return ac.Request()
}
