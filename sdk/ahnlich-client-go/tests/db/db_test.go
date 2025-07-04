package db_test

// import (
// 	"path/filepath"
// 	testing "testing"
// 	"time"

// 	"github.com/stretchr/testify/assert"
// 	"github.com/stretchr/testify/require"

// 	dbQuery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_query"
// 	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"

// 	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
// 	transport "github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
// 	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
// )

// type clientTestFixture struct {
// 	storeName                 string
// 	storeNamePredicate        string
// 	storeDimension            uint64
// 	storePredicates           []string
// 	storeNonLinearAlgorithm   []dbQuery.NonLinearAlgorithm
// 	storeErrorIfExists        bool
// 	storeErrorIfNotExists     bool
// 	stringValueRankPredicate  string
// 	stringValueJobPredicate   string
// 	dropPredicate             string
// 	binaryValueImagePredicate string
// 	stringKeyRank             dbQuery.Array
// 	stringValueRank           map[string]dbQuery.MetadataValue
// 	stringKeyJob              dbQuery.Array
// 	stringValueJob            map[string]dbQuery.MetadataValue
// 	binaryKeyImage            dbQuery.Array
// 	binaryValueImage          map[string]dbQuery.MetadataValue
// 	similaritySearchKey       dbQuery.Array
// }

// type ClientTestSuite struct {
// 	client  *AhnlichDBClient
// 	fixture clientTestFixture
// 	db      *utils.AhnlichDBTestSuite
// }

// type testArgs struct {
// 	name      string
// 	args      []interface{}
// 	want      interface{}
// 	wantErr   bool
// 	caller    func(...interface{}) ([]AhnlichDBResponse, error)
// 	wantMatch bool
// 	parralel  bool // Run the test in parallel
// }

// func loadAndTestFixture(t *testing.T, client *AhnlichDBClient) clientTestFixture {
// 	var tests [][]testArgs

// 	stringValueRankPredicate := "rank"
// 	stringValueJobPredicate := "job"
// 	binaryValueImagePredicate := "image"
// 	dropPredicate := "to_drop"
// 	stringRank := "chunin"
// 	stringJob := "scarvenger"
// 	storeName := "Golang Test Store"
// 	storeNamePredicate := "GoLang Test Store with Predicate"
// 	binaryImage := []uint8{2, 2, 3, 4, 5, 6, 7}
// 	storeDimension := uint64(5)
// 	storePredicates := []string{stringValueRankPredicate, stringValueJobPredicate, binaryValueImagePredicate, dropPredicate}
// 	storeErrorIfExists := true
// 	storeErrorIfNotExists := true
// 	stringKeyRank := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 4.0, 5.0}, 1)
// 	stringValueRank := MakeDBQueryMetaDataType(map[string]string{stringValueRankPredicate: stringRank}) // String
// 	stringKeyJob := MakeDBQueryArrayType([]float32{5.0, 3.0, 4.0, 3.9, 4.9}, 1)
// 	stringValueJob := MakeDBQueryMetaDataType(map[string]string{stringValueJobPredicate: stringJob}) // String
// 	binaryKeyImage := MakeDBQueryArrayType([]float32{1.0, 4.0, 3.0, 3.9, 4.9}, 1)
// 	binaryValueImage := MakeDBQueryMetaDataTypeBinary(map[string][]uint8{binaryValueImagePredicate: binaryImage}) // Binary
// 	similaritySearchKey := MakeDBQueryArrayType([]float32{1.0, 2.0, 3.0, 3.9, 4.9}, 1)
// 	storeNonLinearAlgorithm := []dbQuery.NonLinearAlgorithm{}

// 	// Load and Test Fixtures
// 	create := []testArgs{
// 		{
// 			name:    "CreateStore",
// 			args:    []interface{}{storeName, storeDimension, []string{}, storeNonLinearAlgorithm, storeErrorIfExists},
// 			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__Unit{}}),
// 			wantErr: false,
// 			caller: func(i ...interface{}) ([]AhnlichDBResponse, error) {
// 				return client.CreateStore(
// 					i[0].(string),
// 					i[1].(uint64),
// 					i[2].([]string),
// 					i[3].([]dbQuery.NonLinearAlgorithm),
// 					i[4].(bool),
// 				)
// 			},
// 			wantMatch: false,
// 		},
// 		{
// 			name:    "CreateStorewithPredicate",
// 			args:    []interface{}{storeNamePredicate, storeDimension, storePredicates, storeNonLinearAlgorithm, storeErrorIfExists},
// 			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__Unit{}}),
// 			wantErr: false,
// 			caller: func(i ...interface{}) ([]AhnlichDBResponse, error) {
// 				return client.CreateStore(
// 					i[0].(string),
// 					i[1].(uint64),
// 					i[2].([]string),
// 					i[3].([]dbQuery.NonLinearAlgorithm),
// 					i[4].(bool),
// 				)
// 			},
// 		},
// 	}
// 	get := []testArgs{
// 		{
// 			name: "GetByPredicateBeforeIndexIsCreated",
// 			args: []interface{}{storeName, &dbQuery.PredicateCondition__Value{
// 				Value: &dbQuery.Predicate__Equals{
// 					Key:   stringValueJobPredicate,
// 					Value: stringValueJob[stringValueJobPredicate],
// 				},
// 			}},
// 			want: []AhnlichDBResponse([]AhnlichDBResponse{
// 				[]struct {
// 					Field0 []float32
// 					Field1 dbResponse.MetadataValue
// 				}{
// 					{
// 						Field0: stringKeyJob.Data,
// 						Field1: MakeDBResponseMetaDataType(map[string]string{stringValueJobPredicate: stringJob})[stringValueRankPredicate],
// 					},
// 				},
// 			}),
// 			wantErr: false,
// 			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {

// 				predicateResponse, err := client.GetByPredicate(args[0].(string), args[1].(dbQuery.PredicateCondition))
// 				require.NoError(t, err)
// 				require.NotEmpty(t, predicateResponse)

// 				predicate := predicateResponse[0].(dbResponse.ServerResponse__Get)
// 				require.NotEmpty(t, predicate)
// 				result := []AhnlichDBResponse{}
// 				return append(result, []struct {
// 					Field0 []float32
// 					Field1 dbResponse.MetadataValue
// 				}{
// 					{
// 						Field0: predicate[0].Field0.Data,
// 						Field1: predicate[0].Field1[stringValueRankPredicate],
// 					},
// 				}), nil

// 			},
// 			wantMatch: true,
// 		},
// 	}
// 	set := []testArgs{

// 		{
// 			name: "SetStoreKeysWithStrings",
// 			args: []interface{}{storeName, []struct {
// 				Field0 dbQuery.Array
// 				Field1 map[string]dbQuery.MetadataValue
// 			}{{
// 				Field0: stringKeyJob,
// 				Field1: stringValueJob,
// 			},

// 				{
// 					Field0: stringKeyRank,
// 					Field1: stringValueRank,
// 				},
// 			}},
// 			want: []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.StoreUpsert{
// 				Inserted: 2,
// 				Updated:  0,
// 			}}),
// 			wantErr: false,
// 			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
// 				return client.Set(
// 					args[0].(string),
// 					args[1].([]struct {
// 						Field0 dbQuery.Array
// 						Field1 map[string]dbQuery.MetadataValue
// 					}),
// 				)
// 			},
// 			wantMatch: false,
// 		},
// 		{
// 			name: "SetStoreKeysWithBinary",
// 			args: []interface{}{storeName, []struct {
// 				Field0 dbQuery.Array
// 				Field1 map[string]dbQuery.MetadataValue
// 			}{{
// 				Field0: binaryKeyImage,
// 				Field1: binaryValueImage,
// 			}}},
// 			want: []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.StoreUpsert{
// 				Inserted: 1,
// 				Updated:  0,
// 			}}),
// 			wantErr: false,
// 			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
// 				return client.Set(
// 					args[0].(string),
// 					args[1].([]struct {
// 						Field0 dbQuery.Array
// 						Field1 map[string]dbQuery.MetadataValue
// 					}),
// 				)
// 			},
// 			wantMatch: false,
// 		},
// 	}
// 	others := []testArgs{
// 		{
// 			name:    "CreatePredicateIndex",
// 			args:    []interface{}{storeName, storePredicates},
// 			want:    []AhnlichDBResponse([]AhnlichDBResponse{dbResponse.ServerResponse__CreateIndex(len(storePredicates))}),
// 			wantErr: false,
// 			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
// 				return client.CreatePredicateIndex(
// 					args[0].(string),
// 					args[1].([]string),
// 				)
// 			},
// 			wantMatch: false,
// 		},
// 		{
// 			name: "ListStores",
// 			args: []interface{}{},
// 			want: []AhnlichDBResponse([]AhnlichDBResponse{
// 				dbResponse.StoreInfo{
// 					Name: storeNamePredicate,
// 				},
// 				dbResponse.StoreInfo{
// 					Name: storeName,
// 				},
// 			}),
// 			wantErr: false,
// 			caller: func(args ...interface{}) ([]AhnlichDBResponse, error) {
// 				stores, err := client.ListStores()
// 				require.NoError(t, err)
// 				require.NotEmpty(t, stores)
// 				storeList := stores[0].(dbResponse.ServerResponse__StoreList) // This is the expected response
// 				storeListNames := []AhnlichDBResponse{}
// 				for _, store := range storeList {
// 					storeListNames = append(storeListNames, dbResponse.StoreInfo{
// 						Name: store.Name,
// 					})
// 				}
// 				return storeListNames, nil

// 			},
// 			wantMatch: true,
// 		},
// 	}

// 	tests = append(tests, create)
// 	tests = append(tests, set)
// 	tests = append(tests, get)
// 	tests = append(tests, others)

// 	// Run main tests
// 	for _, test := range tests {
// 		for _, arg := range test {
// 			t.Run(arg.name, func(t *testing.T) {
// 				if arg.parralel {
// 					t.Parallel()
// 				}
// 				response, err := arg.caller(arg.args...)
// 				require.NoError(t, err)
// 				require.NotEmpty(t, response)
// 				if arg.wantMatch {
// 					require.ElementsMatchf(t, response, arg.want, "Expected %v, got %v", arg.want, response)
// 				} else {
// 					require.Equal(t, response, arg.want)
// 				}
// 			})
// 		}
// 	}

// 	return clientTestFixture{
// 		storeName:                 storeName,
// 		storeNamePredicate:        storeNamePredicate,
// 		storeDimension:            storeDimension,
// 		storePredicates:           storePredicates,
// 		storeNonLinearAlgorithm:   storeNonLinearAlgorithm,
// 		storeErrorIfExists:        storeErrorIfExists,
// 		storeErrorIfNotExists:     storeErrorIfNotExists,
// 		stringKeyRank:             stringKeyRank,
// 		stringValueRank:           stringValueRank,
// 		stringKeyJob:              stringKeyJob,
// 		stringValueJob:            stringValueJob,
// 		binaryKeyImage:            binaryKeyImage,
// 		binaryValueImage:          binaryValueImage,
// 		similaritySearchKey:       similaritySearchKey,
// 		stringValueRankPredicate:  stringValueRankPredicate,
// 		stringValueJobPredicate:   stringValueJobPredicate,
// 		binaryValueImagePredicate: binaryValueImagePredicate,
// 		dropPredicate:             dropPredicate,
// 	}
// }

// func newClientTestSuite(t *testing.T, args ...utils.OptionalArgs) *ClientTestSuite {
// 	var dbClient *AhnlichDBClient
// 	t.Cleanup(func() {
// 		if dbClient != nil {
// 			dbClient.Close()
// 		}
// 	})
// 	db := utils.RunAhnlichDatabase(t, args...)
// 	config := ahnlichclientgo.LoadConfig(
// 		ahnlichclientgo.ConnectionConfig{
// 			Host:                   db.Host,
// 			Port:                   db.Port,
// 			InitialConnections:     5,
// 			MaxIdleConnections:     5,
// 			MaxTotalConnections:    5,
// 			ConnectionIdleTimeout:  5,
// 			ReadTimeout:            5 * time.Second,
// 			WriteTimeout:           5 * time.Second,
// 			BackoffMaxElapsedTime:  5 * time.Second,
// 			BackoffInitialInterval: 1 * time.Second,
// 			BackoffMaxInterval:     2 * time.Second,
// 		})

// 	// Initialize the ahnlich database client
// 	cm, err := transport.NewConnectionManager(config.ConnectionConfig)
// 	require.NoError(t, err)

// 	dbClient, err = NewAhnlichDBClient(cm)
// 	require.NoError(t, err)

// 	fixture := loadAndTestFixture(t, dbClient)

// 	return &ClientTestSuite{
// 		client:  dbClient,
// 		fixture: fixture,
// 		db:      db,
// 	}
// }

// func TestClient_GetKeys(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Get keys from the store
// 	getKeys := dbQuery.Query__GetKey{
// 		Store: ts.fixture.storeName,
// 		Keys:  []dbQuery.Array{ts.fixture.stringKeyRank},
// 	}
// 	getKeysResponse, err := ts.client.GetByKeys(getKeys.Store, getKeys.Keys)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, getKeysResponse[0].(dbResponse.ServerResponse__Get))
// 	key := getKeysResponse[0].(dbResponse.ServerResponse__Get)[0]
// 	assert.Equal(t, key.Field0.Data, ts.fixture.stringKeyRank.Data)
// 	assert.EqualValues(t, key.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
// }

// func TestClient_GetByPredicates(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Get by predicate with index
// 	getByPredicateResponse, err := ts.client.GetByPredicate(ts.fixture.storeName, &dbQuery.PredicateCondition__Value{
// 		Value: &dbQuery.Predicate__Equals{
// 			Key:   ts.fixture.stringValueJobPredicate,
// 			Value: ts.fixture.stringValueJob[ts.fixture.stringValueJobPredicate],
// 		},
// 	})
// 	require.NoError(t, err)
// 	require.NotEmpty(t, getByPredicateResponse[0].(dbResponse.ServerResponse__Get))
// 	predicate := getByPredicateResponse[0].(dbResponse.ServerResponse__Get)[0]
// 	assert.Equal(t, predicate.Field0.Data, ts.fixture.stringKeyJob.Data)
// 	assert.EqualValues(t, predicate.Field1[ts.fixture.stringValueJobPredicate], ts.fixture.stringValueJob[ts.fixture.stringValueJobPredicate])
// }

// func TestClient_DropPredicate(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Drop predicate index
// 	dropPredicateIndex, err := ts.client.DropPredicateIndex(ts.fixture.storeName, []string{ts.fixture.dropPredicate}, false)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, dropPredicateIndex)
// 	dropPredicateIndexResponse := dropPredicateIndex[0].(dbResponse.ServerResponse__Del)
// 	assert.Equal(t, dropPredicateIndexResponse, dbResponse.ServerResponse__Del(1))
// }

// func TestClient_DeletePredicate(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Delete predicate
// 	deletePredicate, err := ts.client.DeletePredicate(ts.fixture.storeName, &dbQuery.PredicateCondition__Value{
// 		Value: &dbQuery.Predicate__Equals{
// 			Key:   ts.fixture.stringValueRankPredicate,
// 			Value: ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate],
// 		},
// 	})
// 	require.NoError(t, err)
// 	require.NotEmpty(t, deletePredicate)
// 	deletePredicateResponse := deletePredicate[0].(dbResponse.ServerResponse__Del)
// 	assert.Equal(t, deletePredicateResponse, dbResponse.ServerResponse__Del(1))
// }

// func TestClient_SimilarKey(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Get_sim_n
// 	// ts.fixture.similaritySearchKey should be close to ts.fixture.stringKeyRank; the test should return ts.fixture.stringKeyRank
// 	getSimN := dbQuery.Query__GetSimN{
// 		Store:       ts.fixture.storeName,
// 		SearchInput: ts.fixture.similaritySearchKey,
// 		ClosestN:    1,
// 		Algorithm:   &dbQuery.Algorithm__CosineSimilarity{},
// 		Condition:   nil,
// 	}
// 	getSimNResponse, err := ts.client.GetBySimN(getSimN.Store, getSimN.SearchInput, getSimN.ClosestN, getSimN.Algorithm, getSimN.Condition)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, getSimNResponse[0].(dbResponse.ServerResponse__GetSimN))
// 	simN := getSimNResponse[0].(dbResponse.ServerResponse__GetSimN)[0]
// 	assert.Equal(t, simN.Field0.Data, ts.fixture.stringKeyRank.Data)
// 	assert.EqualValues(t, simN.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
// 	expectedSimilarity := float32(0.9999504)
// 	// assert that the similarity is close to 1 (cosine similarity)
// 	assert.InDelta(t, float32(simN.Field2), expectedSimilarity, 0.0001)
// }

// func TestClient_DeleteKeys(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Delete keys
// 	deleteKeys := dbQuery.Query__DelKey{
// 		Store: ts.fixture.storeName,
// 		Keys:  []dbQuery.Array{ts.fixture.stringKeyJob},
// 	}
// 	deleteKeysResponse, err := ts.client.DeleteKeys(deleteKeys.Store, deleteKeys.Keys)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, deleteKeysResponse)
// 	deleteKeysResponseResult := deleteKeysResponse[0].(dbResponse.ServerResponse__Del)
// 	assert.Equal(t, deleteKeysResponseResult, dbResponse.ServerResponse__Del(1))
// }

// func TestClient_DeleteStore(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Delete the store without predicates
// 	deleteStoreResponse, err := ts.client.DropStore(ts.fixture.storeName, ts.fixture.storeErrorIfNotExists)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, deleteStoreResponse)
// 	deleteStoreResult := deleteStoreResponse[0].(dbResponse.ServerResponse__Del)
// 	require.Equal(t, deleteStoreResult, dbResponse.ServerResponse__Del(1))

// 	// Check if the store exists and list the stores.
// 	stores, err := ts.client.ListStores()
// 	require.NoError(t, err)
// 	require.NotEmpty(t, stores)

// 	store := stores[0].(dbResponse.ServerResponse__StoreList)
// 	require.Equal(t, len(store), 1)
// 	assert.Equal(t, store[0].Name, ts.fixture.storeNamePredicate)

// }

// func TestClient_Ping(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Ping the server
// 	pingResponse, err := ts.client.Ping()
// 	require.NoError(t, err)
// 	require.NotEmpty(t, pingResponse)
// 	assert.Equal(t, pingResponse[0], dbResponse.ServerResponse__Pong{})
// }

// func TestClient_ServerInfo(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Get the server info
// 	serverInfoResponse, err := ts.client.ServerInfo()
// 	require.NoError(t, err)
// 	require.NotEmpty(t, serverInfoResponse)
// 	infoResult := serverInfoResponse[0].(dbResponse.ServerInfo)
// 	protocolVersion := ts.client.ProtocolVersion()
// 	assert.Equal(t, infoResult.Version, protocolVersion)
// 	connectionInfo := ts.client.ConnectionInfo()
// 	assert.Equal(t, infoResult.Address, connectionInfo.remoteAddr)
// }

// func TestClient_ListClients(t *testing.T) {
// 	timeBefore := time.Now()
// 	ts := newClientTestSuite(t)
// 	timeAfter := time.Now()
// 	// List the clients
// 	clientsResponse, err := ts.client.ListClients()
// 	require.NoError(t, err)
// 	require.NotEmpty(t, clientsResponse)
// 	clientsResult := clientsResponse[0].(dbResponse.ServerResponse__ClientList)
// 	assert.NotEmpty(t, clientsResult)
// 	client := clientsResult[0]
// 	connectionInfo := ts.client.ConnectionInfo()
// 	assert.Equal(t, client.Address, connectionInfo.localAddr)
// 	assert.LessOrEqual(t, client.TimeConnected.SecsSinceEpoch, uint64(timeAfter.Unix()))
// 	assert.GreaterOrEqual(t, client.TimeConnected.SecsSinceEpoch, uint64(timeBefore.Unix()))
// }

// func TestClient_Pipeline(t *testing.T) {
// 	ts := newClientTestSuite(t)
// 	// Pipeline
// 	pipeline := ts.client.Pipeline()
// 	pipeline, err := pipeline.BuildPingQuery()
// 	require.NoError(t, err)
// 	pipeline, err = pipeline.BuildInfoServerQuery()
// 	require.NoError(t, err)
// 	pipeline, err = pipeline.BuildListClientsQuery()
// 	require.NoError(t, err)
// 	pipeline, err = pipeline.BuildListStoresQuery() // Ping, ServerInfo, ListClients, ListStores
// 	require.NoError(t, err)
// 	pipelineResponse, err := ts.client.ExecutePipeline(pipeline)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, pipelineResponse)
// 	require.Equal(t, len(pipelineResponse), 4)
// 	// The pong should be the first response
// 	result, ok := pipelineResponse[0].(dbResponse.ServerResponse__Pong)
// 	assert.True(t, ok)
// 	assert.NotNil(t, result)
// 	// The server info should contain the server info
// 	result1, ok := pipelineResponse[1].(dbResponse.ServerInfo)
// 	assert.True(t, ok)
// 	assert.NotEmpty(t, result1)
// 	protocolVersion := ts.client.ProtocolVersion()
// 	assert.Equal(t, result1.Version, protocolVersion)
// 	connectionInfo := ts.client.ConnectionInfo()
// 	assert.Equal(t, result1.Address, connectionInfo.remoteAddr)
// 	// The client list should contain the client that is executing the pipeline
// 	result2, ok := pipelineResponse[2].(dbResponse.ServerResponse__ClientList)
// 	assert.True(t, ok)
// 	assert.NotEmpty(t, result2)
// 	assert.Equal(t, result2[0].Address, connectionInfo.localAddr)
// 	// The store list should contain the store that was created in the test
// 	result3, ok := pipelineResponse[3].(dbResponse.ServerResponse__StoreList)
// 	assert.True(t, ok)
// 	assert.NotEmpty(t, result3)
// 	require.Equal(t, len(result3), 2)
// 	storeNames := []string{result3[0].Name, result3[1].Name}
// 	assert.Contains(t, storeNames, ts.fixture.storeName)
// 	assert.Contains(t, storeNames, ts.fixture.storeNamePredicate)
// }

// func TestDbPersistence(t *testing.T) {
// 	// t.Skip("skipping test")
// 	waitTimeInterval := 3 * time.Second // same as the db persistence interval
// 	// test Db persistence
// 	// start the database with persistence and Load fixtures data into the database
// 	tempDir := t.TempDir()
// 	tempFile := filepath.Join(tempDir, "db_persistence.json")
// 	persistOption := &utils.PersistOption{
// 		Persistence:             true,
// 		PersistenceFileLocation: tempFile,
// 		PersistenceInterval:     100,
// 	}
// 	ts := newClientTestSuite(t, persistOption)
// 	require.True(t, ts.db.IsRunning())
// 	// wait for some time
// 	time.Sleep(waitTimeInterval)
// 	// Stop/kill the database
// 	t.Log(ts.db.StdOut.String())
// 	t.Log(ts.db.StdErr.String())
// 	ts.db.Kill()
// 	require.False(t, ts.db.IsRunning())

// 	// list all files in the persistencelocation
// 	fileList, err := utils.ListFilesInDir(tempDir)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, fileList)
// 	// Check if file is created in the persistencelocation
// 	assert.Contains(t, fileList, utils.GetFileFromPath(tempFile))
// 	utils.ValidateJsonFile(t, tempFile)

// 	// Start the database again on same port and host
// 	addrsOption := &utils.AddrsOption{
// 		ServerAddr: ts.db.ServerAddr,
// 	}
// 	ts.db = utils.RunAhnlichDatabase(t, persistOption, addrsOption)
// 	require.True(t, ts.db.IsRunning())

// 	require.Equal(t, ts.db.ServerAddr, addrsOption.ServerAddr)

// 	// Check if the store data is still present in the database
// 	// List the stores
// 	stores, err := ts.client.ListStores()
// 	require.NoError(t, err)
// 	require.NotEmpty(t, stores)
// 	storeList, ok := stores[0].(dbResponse.ServerResponse__StoreList)
// 	assert.True(t, ok)
// 	assert.NotEmpty(t, storeList)
// 	require.Equal(t, len(storeList), 2)
// 	storeNames := []string{storeList[0].Name, storeList[1].Name}
// 	assert.Contains(t, storeNames, ts.fixture.storeName)
// 	assert.Contains(t, storeNames, ts.fixture.storeNamePredicate)
// 	// Get Keys in Store
// 	getKeys := dbQuery.Query__GetKey{
// 		Store: ts.fixture.storeName,
// 		Keys:  []dbQuery.Array{ts.fixture.stringKeyRank},
// 	}
// 	getKeysResponse, err := ts.client.GetByKeys(getKeys.Store, getKeys.Keys)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, getKeysResponse[0].(dbResponse.ServerResponse__Get))
// 	key := getKeysResponse[0].(dbResponse.ServerResponse__Get)[0]
// 	assert.Equal(t, key.Field0.Data, ts.fixture.stringKeyRank.Data)
// 	assert.EqualValues(t, key.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
// 	// Get Sim N
// 	getSimN := dbQuery.Query__GetSimN{
// 		Store:       ts.fixture.storeName,
// 		SearchInput: ts.fixture.similaritySearchKey,
// 		ClosestN:    1,
// 		Algorithm:   &dbQuery.Algorithm__CosineSimilarity{},
// 		Condition:   nil,
// 	}
// 	getSimNResponse, err := ts.client.GetBySimN(getSimN.Store, getSimN.SearchInput, getSimN.ClosestN, getSimN.Algorithm, getSimN.Condition)
// 	require.NoError(t, err)
// 	require.NotEmpty(t, getSimNResponse[0].(dbResponse.ServerResponse__GetSimN))
// 	simN := getSimNResponse[0].(dbResponse.ServerResponse__GetSimN)[0]
// 	assert.Equal(t, simN.Field0.Data, ts.fixture.stringKeyRank.Data)
// 	assert.EqualValues(t, simN.Field1[ts.fixture.stringValueRankPredicate], ts.fixture.stringValueRank[ts.fixture.stringValueRankPredicate])
// 	expectedSimilarity := float32(0.9999504)
// 	// assert that the similarity is close to 1 (cosine similarity)
// 	assert.InDelta(t, float32(simN.Field2), expectedSimilarity, 0.0001)
// }
