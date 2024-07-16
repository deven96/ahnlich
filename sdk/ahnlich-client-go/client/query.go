package client

import (
	"errors"

	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// AhnlichDBQueryBuilder builds queries based on input parameters
type AhnlichDBQueryBuilder struct {
	queries []dbQuery.Query
}

// NewAhnlichAhnlichQueryBuilder creates a new instance of AhnlichDBQueryBuilder
func NewAhnlichDBQueryBuilder() *AhnlichDBQueryBuilder {
	return &AhnlichDBQueryBuilder{
		queries: make([]dbQuery.Query, 0),
	}
}

func (qb *AhnlichDBQueryBuilder) BuildCreateStoreQuery(storeName string, dimension uint64, predicates []string, nonLinearAlgorithm []dbQuery.NonLinearAlgorithm, errorIfExist bool) error {
	nonZeroDimension, err := utils.NewNonZeroUint(dimension)
	if err != nil {
		return err
	}
	qb.queries = append(qb.queries, &dbQuery.Query__CreateStore{
		Store:            storeName,
		Dimension:        nonZeroDimension.Value,
		CreatePredicates: predicates,
		NonLinearIndices: nonLinearAlgorithm,
		ErrorIfExists:    errorIfExist,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildGetByKeysQuery(storeName string, keys []dbQuery.Array) error {
	qb.queries = append(qb.queries, &dbQuery.Query__GetKey{
		Store: storeName,
		Keys:  keys,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildGetByPredicateQuery(storeName string, condition dbQuery.PredicateCondition) error {
	qb.queries = append(qb.queries, &dbQuery.Query__GetPred{
		Store:     storeName,
		Condition: condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildGetBySimNQuery(storeName string, searchInput dbQuery.Array, closest_n uint64, algorithm dbQuery.Algorithm, condition *dbQuery.PredicateCondition) error {
	nonZeroClosestN, err := utils.NewNonZeroUint(closest_n)
	if err != nil {
		return err
	}
	qb.queries = append(qb.queries, &dbQuery.Query__GetSimN{
		Store:       storeName,
		SearchInput: searchInput,
		ClosestN:    nonZeroClosestN.Value,
		Algorithm:   algorithm,
		Condition:   condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDropPredicateIndexQuery(storeName string, predicates []string, errorIfNotExist bool) error {
	qb.queries = append(qb.queries, &dbQuery.Query__DropPredIndex{
		Store:            storeName,
		Predicates:       predicates,
		ErrorIfNotExists: errorIfNotExist,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildCreatePredicateIndexQuery(storeName string, predicates []string) error {
	qb.queries = append(qb.queries, &dbQuery.Query__CreatePredIndex{
		Store:      storeName,
		Predicates: predicates,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildSetQuery(storeName string, inputs []struct {
	Field0 dbQuery.Array
	Field1 map[string]dbQuery.MetadataValue
}) error {
	qb.queries = append(qb.queries, &dbQuery.Query__Set{
		Store:  storeName,
		Inputs: inputs,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDeleteKeysQuery(storeName string, keys []dbQuery.Array) error {
	qb.queries = append(qb.queries, &dbQuery.Query__DelKey{
		Store: storeName,
		Keys:  keys,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDeletePredicateQuery(storeName string, condition dbQuery.PredicateCondition) error {
	qb.queries = append(qb.queries, &dbQuery.Query__DelPred{
		Store:     storeName,
		Condition: condition,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildDropStoreQuery(storeName string, errorIfNotExist bool) error {
	qb.queries = append(qb.queries, &dbQuery.Query__DropStore{
		Store:            storeName,
		ErrorIfNotExists: errorIfNotExist,
	})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildListStoresQuery() error {
	qb.queries = append(qb.queries, &dbQuery.Query__ListStores{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildInfoServerQuery() error {
	qb.queries = append(qb.queries, &dbQuery.Query__InfoServer{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildListClientsQuery() error {
	qb.queries = append(qb.queries, &dbQuery.Query__ListClients{})
	return nil
}

func (qb *AhnlichDBQueryBuilder) BuildPingQuery() error {
	qb.queries = append(qb.queries, &dbQuery.Query__Ping{})
	return nil
}

// DropQueries drops all the queries in the query builder
func (qb *AhnlichDBQueryBuilder) DropQueries() error {
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
	qb.DropQueries()
	return &dbQuery.ServerQuery{Queries: queries}, nil
}
