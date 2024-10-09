package client

import (
	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
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

// Execute sends the queries in the pipeline to the ahnlich db server and returns the response
func (ac *AhnlichDBClient) Execute() ([]AhnlichDBResponse, error) {
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
func (ac *AhnlichDBClient) ProtocolVersion() dbResponse.Version {
	return ac.version
}

// Version returns the version of the client
func (ac *AhnlichDBClient) Version() dbResponse.Version {
	return ac.clientVersion
}

// ConnectionInfo returns the connection information
func (ac *AhnlichDBClient) ConnectionInfo() *connectionInfo {
	return ac.connectionInfo
}

// Pipeline returns the pipeline for the client
func (ac *AhnlichDBClient) Pipeline() *AhnlichDBQueryBuilder {
	return ac.pipeline
}

func (ac *AhnlichDBClient) Ping() ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildPingQuery()
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

// ExecutePipeline sets the pipeline for the client and sends the request to the server
func (ac *AhnlichDBClient) ExecutePipeline(pipeline *AhnlichDBQueryBuilder) ([]AhnlichDBResponse, error) {
	ac.pipeline = pipeline
	return ac.Execute()
}

func (ac *AhnlichDBClient) ServerInfo() ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildInfoServerQuery()
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) ListClients() ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildListClientsQuery()
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) ListStores() ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildListStoresQuery()
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) CreatePredicateIndex(storeName string, predicates []string) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildCreatePredicateIndexQuery(storeName, predicates)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) CreateStore(storeName string, dimension uint64, predicates []string, nonLinearAlgorithms []dbQuery.NonLinearAlgorithm, errorIfExist bool) ([]AhnlichDBResponse, error) {
	nonZeroDimension, err := utils.NewNonZeroUint(dimension)
	if err != nil {
		return nil, err
	}
	_, err = ac.pipeline.BuildCreateStoreQuery(storeName, nonZeroDimension.Value, predicates, nonLinearAlgorithms, errorIfExist)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) Set(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildSetQuery(storeName, inputs)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) GetByKeys(storeName string, keys []dbQuery.Array) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildGetByKeysQuery(storeName, keys)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) GetByPredicate(storeName string, condition dbQuery.PredicateCondition) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildGetByPredicateQuery(storeName, condition)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) GetBySimN(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) ([]AhnlichDBResponse, error) {
	nonZeroClosestN, err := utils.NewNonZeroUint(closest_n)
	if err != nil {
		return nil, err
	}
	_, err = ac.pipeline.BuildGetBySimNQuery(storeName, searchInput, nonZeroClosestN.Value, algorithm, condition)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) DropPredicateIndex(storeName string, predicates []string, errorIfNotExist bool) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildDropPredicateIndexQuery(storeName, predicates, errorIfNotExist)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) DeleteKeys(storeName string, keys []dbQuery.Array) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildDeleteKeysQuery(storeName, keys)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) DeletePredicate(storeName string, condition dbQuery.PredicateCondition) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildDeletePredicateQuery(storeName, condition)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}

func (ac *AhnlichDBClient) DropStore(storeName string, errorIfNotExist bool) ([]AhnlichDBResponse, error) {
	_, err := ac.pipeline.BuildDropStoreQuery(storeName, errorIfNotExist)
	if err != nil {
		return nil, err
	}
	return ac.Execute()
}
