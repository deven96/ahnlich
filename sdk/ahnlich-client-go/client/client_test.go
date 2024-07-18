package client

import (
	testing "testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	transport "github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

func newDBTestClient(t *testing.T, config ahnlichclientgo.Config) *AhnlichDBClient {
	var dbClient *AhnlichDBClient

	t.Cleanup(func() {
		if dbClient != nil {
			dbClient.Close()
		}
	})

	// Initialize the ahnlich database client
	cm, err := transport.NewConnectionManager(config.ConnectionConfig)
	require.NoError(t, err)

	dbClient, err = NewAhnlichDBClient(cm)
	require.NoError(t, err)

	return dbClient
}

type clientTestFixture struct {
	storeName          string
	storeNamePredicate string
	dimension          uint64
	predicates         []string
	nonLinearAlgorithm []dbQuery.NonLinearAlgorithm
	errorIfExists      bool
	storeKey           dbQuery.Array
	storeValue         map[string]dbQuery.MetadataValue
	storeKey2          dbQuery.Array
	storeValue2        map[string]dbQuery.MetadataValue
	errorIfNotExists   bool
}

func loadFixtures() clientTestFixture {
	storeKey := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 4.0, 5.0}, 1)
	storeKey2 := MakeDBQueryArrayType([]float32{5.0, 3.0, 4.0, 3.9, 4.9}, 1)
	storeValue := MakeDBQueryMetaDataType(map[string]string{"rank": "chuning"})
	storeValue2 := MakeDBQueryMetaDataType(map[string]string{"job": "scarvenger"})
	storeName := "Golang Test"
	storeNamePredicate := "GoLang Test Predicate"
	dimension := uint64(5)
	errorIfExists := true
	predicates := []string{"is_tyrannical", "rank"}
	nonLinearAlgorithm := []dbQuery.NonLinearAlgorithm{}
	errorIfNotExists := true

	return clientTestFixture{
		storeName:          storeName,
		storeNamePredicate: storeNamePredicate,
		dimension:          dimension,
		predicates:         predicates,
		nonLinearAlgorithm: nonLinearAlgorithm,
		errorIfExists:      errorIfExists,
		errorIfNotExists:   errorIfNotExists,
		storeKey:           storeKey,
		storeValue:         storeValue,
		storeKey2:          storeKey2,
		storeValue2:        storeValue2,
	}
}

func TestClient_Store(t *testing.T) {
	// Run the Ahnlich database
	db := utils.RunAhnlichDatabase(t)
	fx := loadFixtures()

	config := ahnlichclientgo.LoadConfig(
		ahnlichclientgo.ConnectionConfig{
			Host:                  db.Host,
			Port:                  db.Port,
			InitialConnections:    1,
			MaxIdleConnections:    1,
			MaxTotalConnections:   1,
			ConnectionIdleTimeout: 5,
			ReadTimeout:           5 * time.Second,
			WriteTimeout:          5 * time.Second,
		})

	client := newDBTestClient(t, config)

	// Create the store with string data and check if the store exists
	createResponse, err := client.CreateStore(fx.storeName, fx.dimension, []string{}, fx.nonLinearAlgorithm, fx.errorIfExists)
	require.NoError(t, err)
	require.NotNil(t, createResponse)
	createSuccessResponse := dbResponse.ServerResponse__Unit{}
	assert.Equal(t, createResponse[0], createSuccessResponse)

	// Check if the store exists and list the stores
	stores, err := client.ListStores()
	assert.NoError(t, err)
	assert.NotNil(t, stores)
	store := stores[0].(dbResponse.ServerResponse__StoreList)
	assert.Equal(t, len(store), 1)
	assert.Equal(t, store[0].Name, fx.storeName)

	// Create the store with predicate and check if the store exists
	respons, err := client.CreateStore(fx.storeNamePredicate, fx.dimension, fx.predicates, fx.nonLinearAlgorithm, fx.errorIfExists)
	require.NoError(t, err)
	require.NotNil(t, respons)
	assert.Equal(t, respons[0], dbResponse.ServerResponse__Unit{})
	// Check if the store exists and list the stores
	stores, err = client.ListStores()
	assert.NoError(t, err)
	assert.NotNil(t, stores)
	store2 := stores[0].(dbResponse.ServerResponse__StoreList)
	assert.Equal(t, len(store2), 2)
	assert.Contains(t, store2, fx.storeNamePredicate)

	// Set the store with string data
	storeData := dbQuery.Query__Set{
		Store: fx.storeName,
		Inputs: []struct {
			Field0 dbQuery.Array
			Field1 map[string]dbQuery.MetadataValue
		}{{
			Field0: fx.storeKey2,
			Field1: fx.storeValue2,
		},
			{
				Field0: fx.storeKey,
				Field1: fx.storeValue,
			},
		},
	}
	storeResponse, err := client.Set(storeData.Store, storeData.Inputs)
	require.NoError(t, err)
	require.NotNil(t, storeResponse)
	setResponse := storeResponse[0].(dbResponse.StoreUpsert)
	storeSuccessResponse := dbResponse.StoreUpsert{
		Inserted: 2,
		Updated:  0,
	}
	assert.Equal(t, setResponse, storeSuccessResponse)

	// Set store with Binary data
	storeData = dbQuery.Query__Set{
		Store: fx.storeName,
		Inputs: []struct {
			Field0 dbQuery.Array
			Field1 map[string]dbQuery.MetadataValue
		}{{
			Field0: MakeDBQueryArrayType([]float32{1.0, 4.0, 3.0, 3.9, 4.9}, 1),
			Field1: MakeDBQueryMetaDataTypeBinary(map[string][]uint8{"image": {2, 2, 3, 4, 5, 6, 7}}),
		},
		},
	}
	storeResponse, err = client.Set(storeData.Store, storeData.Inputs)
	require.NoError(t, err)
	require.NotNil(t, storeResponse)
	setResponse = storeResponse[0].(dbResponse.StoreUpsert)
	storeBinarySuccessResponse := dbResponse.StoreUpsert{
		Inserted: 1,
		Updated:  0,
	}
	assert.Equal(t, setResponse, storeBinarySuccessResponse)

	// Get keys from the store
	getKeys := dbQuery.Query__GetKey{
		Store: fx.storeName,
		Keys:  []dbQuery.Array{fx.storeKey},
	}
	getKeysResponse, err := client.GetByKeys(getKeys.Store, getKeys.Keys)
	require.NoError(t, err)
	require.NotNil(t, getKeysResponse)
	key := getKeysResponse[0].(dbResponse.ServerResponse__Get)[0]
	assert.Equal(t, key.Field0.Data, fx.storeKey.Data)
	assert.EqualValues(t, key.Field1["rank"], fx.storeValue["rank"])

	// Get by predicate without index
	getByPredicate := dbQuery.Query__GetPred{
		Store: fx.storeName,
		Condition: &dbQuery.PredicateCondition__Value{
			Value: &dbQuery.Predicate__Equals{
				Key:   "job",
				Value: fx.storeValue2["job"], // SEgmentation fault here if Value is nil
			},
		},
	}
	getByPredicateResponse, err := client.GetByPredicate(getByPredicate.Store, getByPredicate.Condition)
	require.NoError(t, err)
	require.NotNil(t, getByPredicateResponse)
	predicate := getByPredicateResponse[0].(dbResponse.ServerResponse__Get)[0]
	assert.Equal(t, predicate.Field0.Data, fx.storeKey2.Data)
	assert.EqualValues(t, predicate.Field1["job"], fx.storeValue2["job"])

	// Create predicate index
	createPredicateIndex, err := client.CreatePredicateIndex(fx.storeName, []string{"job", "rank"})
	require.NoError(t, err)
	require.NotNil(t, createPredicateIndex)
	createPredicateIndexResponse := createPredicateIndex[0].(dbResponse.ServerResponse__CreateIndex)
	assert.Equal(t, createPredicateIndexResponse, dbResponse.ServerResponse__CreateIndex(2))

	// Get by predicate with index
	getByPredicate = dbQuery.Query__GetPred{
		Store: fx.storeName,
		Condition: &dbQuery.PredicateCondition__Value{
			Value: &dbQuery.Predicate__Equals{
				Key:   "job",
				Value: fx.storeValue2["job"],
			},
		},
	}
	getByPredicateResponse, err = client.GetByPredicate(getByPredicate.Store, getByPredicate.Condition)
	require.NoError(t, err)
	require.NotNil(t, getByPredicateResponse)
	predicate = getByPredicateResponse[0].(dbResponse.ServerResponse__Get)[0]
	assert.Equal(t, predicate.Field0.Data, fx.storeKey2.Data)
	assert.EqualValues(t, predicate.Field1["job"], fx.storeValue2["job"])

	// Get_sim_n
	searchKey := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 3.9, 4.9}, 1) // This should be close to fx.storeKey; the test should return fx.storeKey
	getSimN := dbQuery.Query__GetSimN{
		Store:       fx.storeName,
		SearchInput: searchKey,
		ClosestN:    1,
		Algorithm:   &dbQuery.Algorithm__CosineSimilarity{},
		Condition:   nil,
	}
	getSimNResponse, err := client.GetBySimN(getSimN.Store, getSimN.SearchInput, getSimN.ClosestN, getSimN.Algorithm, getSimN.Condition)
	require.NoError(t, err)
	require.NotNil(t, getSimNResponse)
	simN := getSimNResponse[0].(dbResponse.ServerResponse__GetSimN)[0]
	assert.Equal(t, simN.Field0.Data, fx.storeKey.Data)
	assert.EqualValues(t, simN.Field1["rank"], fx.storeValue["rank"])
	expectedSimilarity := float32(0.9999504)
	// assert that the similarity is close to 1 (cosine similarity)
	assert.InDelta(t, float32(simN.Field2), expectedSimilarity, 0.0001)

	// Drop predicate index
	// create a predicate index to drop
	createPredicateIndex, err = client.CreatePredicateIndex(fx.storeName, []string{"to_drop"})
	require.NoError(t, err)
	require.NotNil(t, createPredicateIndex)
	createPredicateIndexResponse = createPredicateIndex[0].(dbResponse.ServerResponse__CreateIndex)
	assert.Equal(t, createPredicateIndexResponse, dbResponse.ServerResponse__CreateIndex(1))
	// drop the predicate index
	dropPredicateIndex, err := client.DropPredicateIndex(fx.storeName, []string{"to_drop"}, false)
	require.NoError(t, err)
	require.NotNil(t, dropPredicateIndex)
	dropPredicateIndexResponse := dropPredicateIndex[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, dropPredicateIndexResponse, dbResponse.ServerResponse__Del(1))

	// Delete predicate
	predictateToDelete := &dbQuery.Query__DelPred{
		Store: fx.storeName,
		Condition: &dbQuery.PredicateCondition__Value{
			Value: &dbQuery.Predicate__Equals{
				Key:   "rank",
				Value: fx.storeValue["rank"], // SEgmentation fault here if Value is nil
			},
		},
	}
	deletePredicate, err := client.DeletePredicate(predictateToDelete.Store, predictateToDelete.Condition)
	require.NoError(t, err)
	require.NotNil(t, deletePredicate)
	deletePredicateResponse := deletePredicate[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, deletePredicateResponse, dbResponse.ServerResponse__Del(1))

	// // Delete keys
	// deleteKeys := &dbQuery.Query__DelKey{
	// 	Store: fx.storeName,
	// 	Keys:  []dbQuery.Array{fx.storeKey},
	// }
	// deleteKeysResponse, err := client.DeleteKeys(deleteKeys.Store, deleteKeys.Keys)
	// require.NoError(t, err)
	// require.NotNil(t, deleteKeysResponse)
	// deleteKeysResponseResult := deleteKeysResponse[0].(dbResponse.ServerResponse__Del)
	// assert.Equal(t, deleteKeysResponseResult, dbResponse.ServerResponse__Del(1))

	// Delete the store
	deleteStore, err := client.DropStore(fx.storeName, fx.errorIfNotExists)
	require.NoError(t, err)
	require.NotNil(t, deleteStore)
	deleteStoreResponse := deleteStore[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, deleteStoreResponse, dbResponse.ServerResponse__Del(1))

	// Check if the store exists and list the stores (reflects dropped store)
	stores, err = client.ListStores()
	assert.NoError(t, err)
	assert.NotNil(t, stores)

	stor := stores[0].(dbResponse.ServerResponse__StoreList)
	assert.Equal(t, len(stor), 1)
	assert.Equal(t, stor[0].Name, fx.storeNamePredicate)

}

func TestAhnlichClient(t *testing.T) {
	// Run the Ahnlich database
	db := utils.RunAhnlichDatabase(t)

	config := ahnlichclientgo.LoadConfig(
		ahnlichclientgo.ConnectionConfig{
			Host:                  db.Host,
			Port:                  db.Port,
			InitialConnections:    1,
			MaxIdleConnections:    1,
			MaxTotalConnections:   1,
			ConnectionIdleTimeout: 5,
			ReadTimeout:           5 * time.Second,
			WriteTimeout:          5 * time.Second,
		})

	client := newDBTestClient(t, config)
	info, _ := client.ServerInfo()
	infoResult := info[0].(dbResponse.ServerInfo)
	protoVersion, err := client.ProtocolVersion()
	assert.NoError(t, err)
	assert.Equal(t, infoResult.Version, protoVersion)
	assert.Equal(t, infoResult.Address, config.ServerAddress)

	ping, err := client.Ping()
	assert.NoError(t, err)
	pingResult := ping[0]
	expectedPong := dbResponse.ServerResponse__Pong{}

	assert.Equal(t, pingResult, expectedPong)

	clients, err := client.ListClients()
	assert.NoError(t, err)
	clientsResult := clients[0].(dbResponse.ServerResponse__ClientList)
	assert.NotNil(t, clientsResult[0])
}

// // Logging
// // Exceptions
// // Connection Timeout
// // Main Application Logic
// // Config
// // Tests
