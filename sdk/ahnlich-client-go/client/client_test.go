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

type clientTestFixture struct {
	storeName                 string
	storeNamePredicate        string
	storeDimension            uint64
	storePredicates           []string
	storeNonLinearAlgorithm   []dbQuery.NonLinearAlgorithm
	storeErrorIfExists        bool
	storeErrorIfNotExists     bool
	stringValueRankPredicate  string
	stringValueJobPredicate   string
	dropPredicate             string
	binaryValueImagePredicate string
	stringKeyRank             dbQuery.Array
	stringValueRank           map[string]dbQuery.MetadataValue
	stringKeyJob              dbQuery.Array
	stringValueJob            map[string]dbQuery.MetadataValue
	binaryKeyImage            dbQuery.Array
	binaryValueImage          map[string]dbQuery.MetadataValue
	similaritySearchKey       dbQuery.Array
}

type ClientTestSuite struct {
	client  *AhnlichDBClient
	fixture clientTestFixture
}

type testArgs struct {
	name      string
	args      []interface{}
	want      interface{}
	wantErr   bool
	caller    func(...interface{}) ([]AhnlichDBResponse, error)
	wantMatch bool
	parralel  bool // Run the test in parallel
}

func loadAndTestFixture(t *testing.T, client *AhnlichDBClient) clientTestFixture {
	var tests [][]testArgs

	stringValueRankPredicate := "rank"
	stringValueJobPredicate := "job"
	binaryValueImagePredicate := "image"
	dropPredicate := "to_drop"
	stringKeyRank := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 4.0, 5.0}, 1)
	stringValueRank := MakeDBQueryMetaDataType(map[string]string{stringValueRankPredicate: "chunin"}) // String
	stringKeyJob := MakeDBQueryArrayType([]float32{5.0, 3.0, 4.0, 3.9, 4.9}, 1)
	stringValueJob := MakeDBQueryMetaDataType(map[string]string{stringValueJobPredicate: "scarvenger"}) // String
	binaryKeyImage := MakeDBQueryArrayType([]float32{1.0, 4.0, 3.0, 3.9, 4.9}, 1)
	binaryValueImage := MakeDBQueryMetaDataTypeBinary(map[string][]uint8{binaryValueImagePredicate: {2, 2, 3, 4, 5, 6, 7}}) // Binary
	similaritySearchKey := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 3.9, 4.9}, 1)
	storeName := "Golang Test Store"
	storeNamePredicate := "GoLang Test Store with Predicate"
	storeDimension := uint64(5)
	storePredicates := []string{stringValueRankPredicate, stringValueJobPredicate, binaryValueImagePredicate, dropPredicate}
	storeNonLinearAlgorithm := []dbQuery.NonLinearAlgorithm{}
	storeErrorIfExists := true
	storeErrorIfNotExists := true

	// Load and Test Fixtures
	create := []testArgs{
		{
			name:    "CreateStore",
			args:    []interface{}{storeName, storeDimension, []string{}, storeNonLinearAlgorithm, storeErrorIfExists},
			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__Unit{}}),
			wantErr: false,
			caller: func(i ...interface{}) ([]AhnlichDBResponse, error) {
				return client.CreateStore(
					i[0].(string),
					i[1].(uint64),
					i[2].([]string),
					i[3].([]dbQuery.NonLinearAlgorithm),
					i[4].(bool),
				)
			},
			wantMatch: false,
		},
		{
			name:    "CreateStorewithPredicate",
			args:    []interface{}{storeNamePredicate, storeDimension, storePredicates, storeNonLinearAlgorithm, storeErrorIfExists},
			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__Unit{}}),
			wantErr: false,
			caller: func(i ...interface{}) ([]AhnlichDBResponse, error) {
				return client.CreateStore(
					i[0].(string),
					i[1].(uint64),
					i[2].([]string),
					i[3].([]dbQuery.NonLinearAlgorithm),
					i[4].(bool),
				)
			},
		},
	}
	get := []testArgs{
		{
			name: "GetByPredicateBeforeIndexIsCreated",
			args: []interface{}{storeName, &dbQuery.PredicateCondition__Value{
				Value: &dbQuery.Predicate__Equals{
					Key:   stringValueJobPredicate,
					Value: stringValueJob[stringValueJobPredicate],
				},
			}},
			want: []AhnlichDBResponse([]AhnlichDBResponse{
				[]struct {
					Field0 []float32
					Field1 dbResponse.MetadataValue
				}{
					{
						Field0: stringKeyJob.Data,
						Field1: MakeDBResponseMetaDataType(map[string]string{stringValueJobPredicate: "scarvenger"})[stringValueRankPredicate],
					},
				},
			}),
			wantErr: false,
			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {

				predicateResponse, err := client.GetByPredicate(args[0].(string), args[1].(dbQuery.PredicateCondition))
				require.NoError(t, err)
				require.NotNil(t, predicateResponse)
				require.NotEmpty(t, predicateResponse)

				predicate := predicateResponse[0].(dbResponse.ServerResponse__Get)
				require.NotNil(t, predicate)
				require.NotEmpty(t, predicate)
				result := []AhnlichDBResponse{}
				return append(result, []struct {
					Field0 []float32
					Field1 dbResponse.MetadataValue
				}{
					{
						Field0: predicate[0].Field0.Data,
						Field1: predicate[0].Field1[stringValueRankPredicate],
					},
				}), nil

			},
			wantMatch: true,
		},
	}
	set := []testArgs{

		{
			name: "SetStoreKeysWithStrings",
			args: []interface{}{storeName, []struct {
				Field0 dbQuery.Array
				Field1 map[string]dbQuery.MetadataValue
			}{{
				Field0: stringKeyJob,
				Field1: stringValueJob,
			},

				{
					Field0: stringKeyRank,
					Field1: stringValueRank,
				},
			}},
			want: []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.StoreUpsert{
				Inserted: 2,
				Updated:  0,
			}}),
			wantErr: false,
			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
				return client.Set(
					args[0].(string),
					args[1].([]struct {
						Field0 dbQuery.Array
						Field1 map[string]dbQuery.MetadataValue
					}),
				)
			},
			wantMatch: false,
		},
		{
			name: "SetStoreKeysWithBinary",
			args: []interface{}{storeName, []struct {
				Field0 dbQuery.Array
				Field1 map[string]dbQuery.MetadataValue
			}{{
				Field0: binaryKeyImage,
				Field1: binaryValueImage,
			}}},
			want: []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.StoreUpsert{
				Inserted: 1,
				Updated:  0,
			}}),
			wantErr: false,
			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
				return client.Set(
					args[0].(string),
					args[1].([]struct {
						Field0 dbQuery.Array
						Field1 map[string]dbQuery.MetadataValue
					}),
				)
			},
			wantMatch: false,
		},
	}
	others := []testArgs{
		{
			name:    "CreatePredicateIndex",
			args:    []interface{}{storeName, storePredicates},
			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__CreateIndex(len(storePredicates))}),
			wantErr: false,
			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
				return client.CreatePredicateIndex(
					args[0].(string),
					args[1].([]string),
				)
			},
			wantMatch: false,
		},
		{
			name: "ListStores",
			args: []interface{}{},
			want: []AhnlichDBResponse([]AhnlichDBResponse{
				dbResponse.StoreInfo{
					Name: storeNamePredicate,
				},
				dbResponse.StoreInfo{
					Name: storeName,
				},
			}),
			wantErr: false,
			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
				stores, err := client.ListStores()
				require.NoError(t, err)
				require.NotNil(t, stores)
				require.NotEmpty(t, stores)
				storeList := stores[0].(dbResponse.ServerResponse__StoreList) // This is the expected response
				storeListNames := []AhnlichDBResponse{}
				for _, store := range storeList {
					storeListNames = append(storeListNames, dbResponse.StoreInfo{
						Name: store.Name,
					})
				}
				return storeListNames, nil

			},
			wantMatch: true,
		},
	}

	tests = append(tests, create)
	tests = append(tests, set)
	tests = append(tests, get)
	tests = append(tests, others)

	// Run main tests
	for _, test := range tests {
		for _, arg := range test {
			t.Run(arg.name, func(t *testing.T) {
				if arg.parralel {
					t.Parallel()
				}
				response, err := arg.caller(arg.args...)
				require.NoError(t, err)
				require.NotNil(t, response)
				if arg.wantMatch {
					require.ElementsMatchf(t, response, arg.want, "Expected %v, got %v", arg.want, response)
				} else {
					require.Equal(t, response, arg.want)
				}
			})
		}
	}

	return clientTestFixture{
		storeName:                 storeName,
		storeNamePredicate:        storeNamePredicate,
		storeDimension:            storeDimension,
		storePredicates:           storePredicates,
		storeNonLinearAlgorithm:   storeNonLinearAlgorithm,
		storeErrorIfExists:        storeErrorIfExists,
		storeErrorIfNotExists:     storeErrorIfNotExists,
		stringKeyRank:             stringKeyRank,
		stringValueRank:           stringValueRank,
		stringKeyJob:              stringKeyJob,
		stringValueJob:            stringValueJob,
		binaryKeyImage:            binaryKeyImage,
		binaryValueImage:          binaryValueImage,
		similaritySearchKey:       similaritySearchKey,
		stringValueRankPredicate:  stringValueRankPredicate,
		stringValueJobPredicate:   stringValueJobPredicate,
		binaryValueImagePredicate: binaryValueImagePredicate,
		dropPredicate:             dropPredicate,
	}
}

func newClientTestSuite(t *testing.T) *ClientTestSuite {
	var dbClient *AhnlichDBClient
	t.Cleanup(func() {
		if dbClient != nil {
			dbClient.Close()
		}
	})
	db := utils.RunAhnlichDatabase(t)
	config := ahnlichclientgo.LoadConfig(
		ahnlichclientgo.ConnectionConfig{
			Host:                  db.Host,
			Port:                  db.Port,
			InitialConnections:    5,
			MaxIdleConnections:    10,
			MaxTotalConnections:   10,
			ConnectionIdleTimeout: 5,
			ReadTimeout:           5 * time.Second,
			WriteTimeout:          5 * time.Second,
		})

	// Initialize the ahnlich database client
	cm, err := transport.NewConnectionManager(config.ConnectionConfig)
	require.NoError(t, err)

	dbClient, err = NewAhnlichDBClient(cm)
	require.NoError(t, err)

	fixture := loadAndTestFixture(t, dbClient)

	return &ClientTestSuite{
		client:  dbClient,
		fixture: fixture,
	}
}

func TestClient_GetKeys(t *testing.T) {
	ts := newClientTestSuite(t)

	// Get keys from the store
	getKeys := dbQuery.Query__GetKey{
		Store: ts.fixture.storeName,
		Keys:  []dbQuery.Array{ts.fixture.stringKeyRank},
	}
	getKeysResponse, err := ts.client.GetByKeys(getKeys.Store, getKeys.Keys)
	require.NoError(t, err)
	require.NotNil(t, getKeysResponse)
	require.NotEmpty(t, getKeysResponse[0].(dbResponse.ServerResponse__Get))
	key := getKeysResponse[0].(dbResponse.ServerResponse__Get)[0]
	assert.Equal(t, key.Field0.Data, ts.fixture.stringKeyRank.Data)
	assert.EqualValues(t, key.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
}

func TestClient_GetByPredicates(t *testing.T) {
	ts := newClientTestSuite(t)

	// Get by predicate with index
	getByPredicate := dbQuery.Query__GetPred{
		Store: ts.fixture.storeName,
		Condition: &dbQuery.PredicateCondition__Value{
			Value: &dbQuery.Predicate__Equals{
				Key:   ts.fixture.stringValueJobPredicate,
				Value: ts.fixture.stringValueJob[ts.fixture.stringValueJobPredicate],
			},
		},
	}
	getByPredicateResponse, err := ts.client.GetByPredicate(getByPredicate.Store, getByPredicate.Condition)
	require.NoError(t, err)
	require.NotNil(t, getByPredicateResponse)
	require.NotEmpty(t, getByPredicateResponse[0].(dbResponse.ServerResponse__Get))
	predicate := getByPredicateResponse[0].(dbResponse.ServerResponse__Get)[0]
	assert.Equal(t, predicate.Field0.Data, ts.fixture.stringKeyJob.Data)
	assert.EqualValues(t, predicate.Field1[ts.fixture.stringValueJobPredicate], ts.fixture.stringValueJob[ts.fixture.stringValueJobPredicate])
}

func TestClient_DropPredicate(t *testing.T) {
	ts := newClientTestSuite(t)
	// Drop predicate index
	dropPredicateIndex, err := ts.client.DropPredicateIndex(ts.fixture.storeName, []string{ts.fixture.dropPredicate}, false)
	require.NoError(t, err)
	require.NotNil(t, dropPredicateIndex)
	dropPredicateIndexResponse := dropPredicateIndex[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, dropPredicateIndexResponse, dbResponse.ServerResponse__Del(1))
}

func TestClient_DeletePredicate(t *testing.T) {
	ts := newClientTestSuite(t)
	// Delete predicate
	predictateToDelete := &dbQuery.Query__DelPred{
		Store: ts.fixture.storeName,
		Condition: &dbQuery.PredicateCondition__Value{
			Value: &dbQuery.Predicate__Equals{
				Key:   ts.fixture.stringValueRankPredicate,
				Value: ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate], // Segmentation fault here if Value is nil
			},
		},
	}
	deletePredicate, err := ts.client.DeletePredicate(predictateToDelete.Store, predictateToDelete.Condition)
	require.NoError(t, err)
	require.NotNil(t, deletePredicate)
	deletePredicateResponse := deletePredicate[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, deletePredicateResponse, dbResponse.ServerResponse__Del(1))
}

func TestClient_SimilarKey(t *testing.T) {
	ts := newClientTestSuite(t)
	// Get_sim_n
	// ts.fixture.similaritySearchKey should be close to ts.fixture.stringKeyRank; the test should return ts.fixture.stringKeyRank
	getSimN := dbQuery.Query__GetSimN{
		Store:       ts.fixture.storeName,
		SearchInput: ts.fixture.similaritySearchKey,
		ClosestN:    1,
		Algorithm:   &dbQuery.Algorithm__CosineSimilarity{},
		Condition:   nil,
	}
	getSimNResponse, err := ts.client.GetBySimN(getSimN.Store, getSimN.SearchInput, getSimN.ClosestN, getSimN.Algorithm, getSimN.Condition)
	require.NoError(t, err)
	require.NotNil(t, getSimNResponse)
	require.NotEmpty(t, getSimNResponse[0].(dbResponse.ServerResponse__GetSimN))
	simN := getSimNResponse[0].(dbResponse.ServerResponse__GetSimN)[0]
	assert.Equal(t, simN.Field0.Data, ts.fixture.stringKeyRank.Data)
	assert.EqualValues(t, simN.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
	expectedSimilarity := float32(0.9999504)
	// assert that the similarity is close to 1 (cosine similarity)
	assert.InDelta(t, float32(simN.Field2), expectedSimilarity, 0.0001)
}

func TestClient_DeleteKeys(t *testing.T) {
	ts := newClientTestSuite(t)
	// Delete keys
	deleteKeys := dbQuery.Query__DelKey{
		Store: ts.fixture.storeName,
		Keys:  []dbQuery.Array{ts.fixture.stringKeyJob},
	}
	deleteKeysResponse, err := ts.client.DeleteKeys(deleteKeys.Store, deleteKeys.Keys)
	require.NoError(t, err)
	require.NotNil(t, deleteKeysResponse)
	deleteKeysResponseResult := deleteKeysResponse[0].(dbResponse.ServerResponse__Del)
	assert.Equal(t, deleteKeysResponseResult, dbResponse.ServerResponse__Del(1))
}

func TestClient_DeleteStore(t *testing.T) {
	ts := newClientTestSuite(t)
	// Delete the store without predicates
	deleteStoreResponse, err := ts.client.DropStore(ts.fixture.storeName, ts.fixture.storeErrorIfNotExists)
	require.NoError(t, err)
	require.NotNil(t, deleteStoreResponse)
	deleteStoreResult := deleteStoreResponse[0].(dbResponse.ServerResponse__Del)
	require.Equal(t, deleteStoreResult, dbResponse.ServerResponse__Del(1))

	// Check if the store exists and list the stores.
	stores, err := ts.client.ListStores()
	require.NoError(t, err)
	require.NotNil(t, stores)
	require.NotEmpty(t, stores)

	store := stores[0].(dbResponse.ServerResponse__StoreList)
	require.Equal(t, len(store), 1)
	assert.Equal(t, store[0].Name, ts.fixture.storeNamePredicate)

}

func TestClient_Ping(t *testing.T) {
	ts := newClientTestSuite(t)
	// Ping the server
	pingResponse, err := ts.client.Ping()
	require.NoError(t, err)
	require.NotNil(t, pingResponse)
	require.NotEmpty(t, pingResponse)
	assert.Equal(t, pingResponse[0], dbResponse.ServerResponse__Pong{})
}

func TestClient_ServerInfo(t *testing.T) {
	ts := newClientTestSuite(t)
	// Get the server info
	serverInfoResponse, err := ts.client.ServerInfo()
	require.NoError(t, err)
	require.NotNil(t, serverInfoResponse)
	require.NotEmpty(t, serverInfoResponse)
	infoResult := serverInfoResponse[0].(dbResponse.ServerInfo)
	protocolVersion, err := ts.client.ProtocolVersion()
	assert.NoError(t, err)
	assert.Equal(t, infoResult.Version, protocolVersion)
	connectionInfo := ts.client.ConnectionInfo()
	assert.Equal(t, infoResult.Address, connectionInfo.remoteAddr)
}

func TestClient_ListClients(t *testing.T) {
	timeBefore := time.Now()
	ts := newClientTestSuite(t)
	timeAfter := time.Now()
	// List the clients
	clientsResponse, err := ts.client.ListClients()
	require.NoError(t, err)
	require.NotNil(t, clientsResponse)
	require.NotEmpty(t, clientsResponse)
	clientsResult := clientsResponse[0].(dbResponse.ServerResponse__ClientList)
	assert.NotNil(t, clientsResult)
	assert.NotEmpty(t, clientsResult)
	client := clientsResult[0]
	connectionInfo := ts.client.ConnectionInfo()
	assert.Equal(t, client.Address, connectionInfo.localAddr)
	assert.LessOrEqual(t, client.TimeConnected.SecsSinceEpoch, uint64(timeAfter.Unix()))
	assert.GreaterOrEqual(t, client.TimeConnected.SecsSinceEpoch, uint64(timeBefore.Unix()))
}

func TestClient_Pipeline(t *testing.T) {
	ts := newClientTestSuite(t)
	// Pipeline
	pipeline := ts.client.Pipeline()
	pipeline.BuildPingQuery().BuildInfoServerQuery().BuildListClientsQuery().BuildListStoresQuery() // Ping, ServerInfo, ListClients, ListStores
	pipelineResponse, err := ts.client.ExecutePipeline(pipeline)
	require.NoError(t, err)
	require.NotNil(t, pipelineResponse)
	require.NotEmpty(t, pipelineResponse)
	require.Equal(t, len(pipelineResponse), 4)
	// The pong should be the first response
	result, ok := pipelineResponse[0].(dbResponse.ServerResponse__Pong)
	assert.True(t, ok)
	assert.NotNil(t, result)
	// The server info should contain the server info
	result1, ok := pipelineResponse[1].(dbResponse.ServerInfo)
	assert.True(t, ok)
	assert.NotNil(t, result1)
	assert.NotEmpty(t, result1)
	protocolVersion, err := ts.client.ProtocolVersion()
	assert.NoError(t, err)
	assert.Equal(t, result1.Version, protocolVersion)
	connectionInfo := ts.client.ConnectionInfo()
	assert.Equal(t, result1.Address, connectionInfo.remoteAddr)
	// The client list should contain the client that is executing the pipeline
	result2, ok := pipelineResponse[2].(dbResponse.ServerResponse__ClientList)
	assert.True(t, ok)
	assert.NotNil(t, result2)
	assert.NotEmpty(t, result2)
	assert.Equal(t, result2[0].Address, connectionInfo.localAddr)
	// The store list should contain the store that was created in the test
	result3, ok := pipelineResponse[3].(dbResponse.ServerResponse__StoreList)
	assert.True(t, ok)
	assert.NotNil(t, result3)
	assert.NotEmpty(t, result3)
	assert.Equal(t, len(result3), 2)
	storeNames := []string{result3[0].Name, result3[1].Name}
	assert.Contains(t, storeNames, ts.fixture.storeName)
	assert.Contains(t, storeNames, ts.fixture.storeNamePredicate)
}
