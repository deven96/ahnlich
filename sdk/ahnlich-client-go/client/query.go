package client

import (
	"errors"
	"fmt"
	"sync"

	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
)

// TODO: Add Validation to the queries to avoid nil pointers

// AhnlichDBQueryBuilder builds queries based on input parameters
type AhnlichDBQueryBuilder struct {
	queries []dbQuery.Query
	mu      *sync.Mutex // To Ensure FIFO order of queries in the pipeline
}

// NewAhnlichAhnlichQueryBuilder creates a new instance of AhnlichDBQueryBuilder
func NewAhnlichDBQueryBuilder() *AhnlichDBQueryBuilder {
	return &AhnlichDBQueryBuilder{
		queries: make([]dbQuery.Query, 0),
		mu:      &sync.Mutex{},
	}
}

func (qb *AhnlichDBQueryBuilder) AddQuery(q dbQuery.Query) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("panic occurred when adding Query to query builder: %v", r) // Convert to AhnlichQueryBuilderException
		}
	}()
	qb.mu.Lock()
	defer qb.mu.Unlock()
	qb.queries = append(qb.queries, q)
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildCreateStoreQuery(storeName string, dimension uint64, predicates []string, nonLinearAlgorithm []dbQuery.NonLinearAlgorithm, errorIfExist bool) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__CreateStore{
		Store:            storeName,
		Dimension:        dimension,
		CreatePredicates: predicates,
		NonLinearIndices: nonLinearAlgorithm,
		ErrorIfExists:    errorIfExist,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildGetByKeysQuery(storeName string, keys []dbQuery.Array) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__GetKey{
		Store: storeName,
		Keys:  keys,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildGetByPredicateQuery(storeName string, condition dbQuery.PredicateCondition) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__GetPred{
		Store:     storeName,
		Condition: condition,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildGetBySimNQuery(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__GetSimN{
		Store:       storeName,
		SearchInput: searchInput,
		ClosestN:    closest_n,
		Algorithm:   algorithm,
		Condition:   condition,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildDropPredicateIndexQuery(storeName string, predicates []string, errorIfNotExist bool) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__DropPredIndex{
		Store:            storeName,
		Predicates:       predicates,
		ErrorIfNotExists: errorIfNotExist,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildCreatePredicateIndexQuery(storeName string, predicates []string) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__CreatePredIndex{
		Store:      storeName,
		Predicates: predicates,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildSetQuery(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__Set{
		Store:  storeName,
		Inputs: inputs,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildDeleteKeysQuery(storeName string, keys []dbQuery.Array) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__DelKey{
		Store: storeName,
		Keys:  keys,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildDeletePredicateQuery(storeName string, condition dbQuery.PredicateCondition) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__DelPred{
		Store:     storeName,
		Condition: condition,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildDropStoreQuery(storeName string, errorIfNotExist bool) (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__DropStore{
		Store:            storeName,
		ErrorIfNotExists: errorIfNotExist,
	})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildListStoresQuery() (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__ListStores{})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildInfoServerQuery() (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__InfoServer{})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildListClientsQuery() (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__ListClients{})
	return qb, err
}

func (qb *AhnlichDBQueryBuilder) BuildPingQuery() (*AhnlichDBQueryBuilder, error) {
	err := qb.AddQuery(&dbQuery.Query__Ping{})
	return qb, err
}

// Reset drops all the queries in the query builder
func (qb *AhnlichDBQueryBuilder) Reset() error {
	qb.queries = make([]dbQuery.Query, 0)
	return nil
}

// ParseBuildQueryToServer parses the queries and builds a server query and drops the queries from the query builder
func (qb *AhnlichDBQueryBuilder) ParseBuildQueryToServerQuery() (*dbQuery.ServerQuery, error) {
	if len(qb.queries) == 0 {
		return nil, errors.New("must have atleast one request to be processed")
	}
	queries := make([]dbQuery.Query, len(qb.queries))
	copy(queries, qb.queries)
	qb.Reset()
	return &dbQuery.ServerQuery{Queries: queries}, nil
}

// Create a DB query array type from a slice of float32
func MakeDBQueryArrayType(data []float32, v uint8) dbQuery.Array {
	data32 := make([]float32, len(data))
	for i, d := range data {
		data32[i] = float32(d)
	}
	dimensions := struct{ Field0 uint64 }{Field0: uint64(len(data))}
	return dbQuery.Array{
		V:    v,
		Dim:  dimensions,
		Data: data32,
	}
}

// Create a DB query map of metadata type from a map of string
func MakeDBQueryMetaDataType(data map[string]string) map[string]dbQuery.MetadataValue {
	metadata := make(map[string]dbQuery.MetadataValue)
	for k, v := range data {
		val := dbQuery.MetadataValue__RawString(v)
		metadata[k] = &val
	}
	return metadata
}

// Create a DB query map of metadata type from a map of binary data in the form of a slice of uint8
func MakeDBQueryMetaDataTypeBinary(data map[string][]uint8) map[string]dbQuery.MetadataValue {
	metadata := make(map[string]dbQuery.MetadataValue)
	for k, v := range data {
		val := dbQuery.MetadataValue__Binary(v)
		metadata[k] = &val
	}
	return metadata
}
