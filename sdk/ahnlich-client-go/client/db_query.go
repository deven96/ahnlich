package client

import (
	"errors"

	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/query"
)

// AhnlichDBQueryBuilder builds Queries based on input parameters
type AhnlichDBQueryBuilder struct {
	Queries []dbQuery.Query
}

// NonZeroUintStruct holds a uint that must be non-zero
type NonZeroUint struct {
	Value uint64
}

// NewNonZeroUint creates a new NonZeroUint ensuring the value is non-zero
func NewNonZeroUint(value uint64) (*NonZeroUint, error) {
	if value == 0 {
		return nil, errors.New("value cannot be zero")
	}
	return &NonZeroUint{Value: value}, nil
}

// NewAhnlichAhnlichQueryBuilder creates a new instance of AhnlichDBQueryBuilder
func NewAhnlichDBQueryBuilder() *AhnlichDBQueryBuilder {
	return &AhnlichDBQueryBuilder{
		Queries: make([]dbQuery.Query, 0),
	}
}

// func (qb *AhnlichDBQueryBuilder) BuildCreateStoreQuery(storeName string, dimension uint64, predicates []string,nonLinearAlgorithm []dbQuery.NonLinearAlgorithm,errorIfExist bool)  error{
// 	nonZeroDimension, err := NewNonZeroUint(dimension)
// 	if err != nil {
// 		return err
// 	}
//     qb.Queries = append(qb.Queries, dbQuery.Query__CreateStore{
// 		Store: storeName,
// 		Dimension: nonZeroDimension.Value,
// 		CreatePredicates: predicates,
// 		NonLinearIndices: nonLinearAlgorithm,
// 		ErrorIfExists: errorIfExist,
// 	})
// 	return nil
// }

func (qb *AhnlichDBQueryBuilder) BuildGetByKeysQuery(storeName string, keys []dbQuery.Array) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__GetKey{
		Store: storeName,
		Keys:  keys,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildGetByPredicateQuery(storeName string, condition dbQuery.PredicateCondition) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__GetPred{
		Store:     storeName,
		Condition: condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildGetSimNQuery(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) error {
	nonZeroClosestN, err := NewNonZeroUint(closest_n)
	if err != nil {
		return err
	}
	qb.Queries = append(qb.Queries, &dbQuery.Query__GetSimN{
		Store:       storeName,
		SearchInput: searchInput,
		ClosestN:    nonZeroClosestN.Value,
		Algorithm:   algorithm,
		Condition:   condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDropPredicateIndexQuery(storeName string, predicates []string, errorIfNotExist bool) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__DropPredIndex{
		Store:            storeName,
		Predicates:       predicates,
		ErrorIfNotExists: errorIfNotExist,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildCreatePredicateIndexQuery(storeName string, predicates []string) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__CreatePredIndex{
		Store:      storeName,
		Predicates: predicates,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildSetQuery(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__Set{
		Store:  storeName,
		Inputs: inputs,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDeleteKeysQuery(storeName string, keys []dbQuery.Array) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__DelKey{
		Store: storeName,
		Keys:  keys,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDeletePredicateQuery(storeName string, condition dbQuery.PredicateCondition) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__DelPred{
		Store:     storeName,
		Condition: condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDropStoreQuery(storeName string, errorIfNotExist bool) error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__DropStore{
		Store:            storeName,
		ErrorIfNotExists: errorIfNotExist,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildListStoresQuery() error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__ListStores{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildInfoServerQuery() error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__InfoServer{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildListClientsQuery() error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__ListClients{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildPingQuery() error {
	qb.Queries = append(qb.Queries, &dbQuery.Query__Ping{})
	return nil
}

// DropQueries drops all the queries in the query builder
func (qb *AhnlichDBQueryBuilder) DropQueries() error {
	qb.Queries = make([]dbQuery.Query, 0)
	return nil
}

// ParseBuildQueryToServer parses the Queries and builds a server query and drops the queries from the query builder
func (qb *AhnlichDBQueryBuilder) ParseBuildQueryToServer() (*dbQuery.ServerQuery, error) {
	if len(qb.Queries) == 0 {
		return nil, errors.New("must have atleast one request to be processed")
	}
	Queries := make([]dbQuery.Query, len(qb.Queries))
	copy(Queries, qb.Queries)
	qb.DropQueries()
	return &dbQuery.ServerQuery{Queries: Queries}, nil
}
