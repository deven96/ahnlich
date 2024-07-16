package client

import (
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
)

// AhnlichDBClient is the client for the Ahnlich DB server
type AhnlichDBClient struct {
	pipeline *AhnlichDBQueryBuilder
	*ahnlichProtocol
}

// NewAhnlichClient creates a new instance of AhnlichClient
func NewAhnlichDBClient(cm *transport.ConnectionManager) (*AhnlichDBClient, error) {
	protocol, err := newAhnlichProtocol(cm)
	if err != nil {
		return nil, err
	}
	return &AhnlichDBClient{
		pipeline:        NewAhnlichDBQueryBuilder(),
		ahnlichProtocol: protocol,
	}, nil
}

// Request sends the queries in the pipeline to the ahnlich db server and returns the response
func (ac *AhnlichDBClient) Request() ([]dbResponse.ServerResponse, error) {
	serverQuery, err := ac.pipeline.ParseBuildQueryToServerQuery()
	if err != nil {
		return nil, err
	}
	response, err := ac.request(serverQuery)
	if err != nil {
		return nil, err
	}
	resp, err := ParseDBResponse(response)
	if err != nil {
		return nil, err
	}
	return resp, nil
}

// Close closes the connection to the server
func (ac *AhnlichDBClient) Close() {
	ac.close()
}

// ProtocolVersion returns the version of the server protocol
func (ac *AhnlichDBClient) ProtocolVersion() (dbResponse.Version, error) {
	return ac.version, nil
}

// Version returns the version of the client
func (ac *AhnlichDBClient) Version() (dbResponse.Version, error) {
	return ac.clientVersion, nil
}

func (ac *AhnlichDBClient) Ping() ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildPingQuery()
	return ac.Request()
}

// Pipeline returns the pipeline for the client
func (ac *AhnlichDBClient) Pipeline() *AhnlichDBQueryBuilder {
	return ac.pipeline
}

// ExecutePipeline sets the pipeline for the client and sends the request to the server
func (ac *AhnlichDBClient) ExecutePipeline(pipeline *AhnlichDBQueryBuilder) ([]dbResponse.ServerResponse, error) {
	ac.pipeline = pipeline
	return ac.Request()
}

func (ac *AhnlichDBClient) ServerInfo() ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildInfoServerQuery()
	return ac.Request()
}

func (ac *AhnlichDBClient) ListClients() ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildListClientsQuery()
	return ac.Request()
}

func (ac *AhnlichDBClient) ListStores() ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildListStoresQuery()
	return ac.Request()
}

func (ac *AhnlichDBClient) CreatePredicateIndex(storeName string, predicates []string) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildCreatePredicateIndexQuery(storeName, predicates)
	return ac.Request()
}

func (ac *AhnlichDBClient) CreateStore(storeName string, dimension uint64, predicates []string, nonLinearAlgorithm []dbQuery.NonLinearAlgorithm, errorIfExist bool) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildCreateStoreQuery(storeName, dimension, predicates, nonLinearAlgorithm, errorIfExist)
	return ac.Request()
}

func (ac *AhnlichDBClient) Set(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildSetQuery(storeName, inputs)
	return ac.Request()
}

func (ac *AhnlichDBClient) GetByKeys(storeName string, keys []dbQuery.Array) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildGetByKeysQuery(storeName, keys)
	return ac.Request()
}

func (ac *AhnlichDBClient) GetByPredicate(storeName string, condition dbQuery.PredicateCondition) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildGetByPredicateQuery(storeName, condition)
	return ac.Request()
}

func (ac *AhnlichDBClient) GetBySimN(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildGetBySimNQuery(storeName, searchInput, closest_n, algorithm, condition)
	return ac.Request()
}

func (ac *AhnlichDBClient) DropPredicateIndex(storeName string, predicates []string, errorIfNotExist bool) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildDropPredicateIndexQuery(storeName, predicates, errorIfNotExist)
	return ac.Request()
}

func (ac *AhnlichDBClient) DeleteKeys(storeName string, keys []dbQuery.Array) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildDeleteKeysQuery(storeName, keys)
	return ac.Request()
}

func (ac *AhnlichDBClient) DeletePredicate(storeName string, condition dbQuery.PredicateCondition) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildDeletePredicateQuery(storeName, condition)
	return ac.Request()
}

func (ac *AhnlichDBClient) DropStore(storeName string, errorIfNotExist bool) ([]dbResponse.ServerResponse, error) {
	ac.pipeline.BuildDropStoreQuery(storeName, errorIfNotExist)
	return ac.Request()
}
