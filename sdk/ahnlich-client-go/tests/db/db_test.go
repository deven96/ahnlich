package db_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"

	nonlinear "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
	pipeline "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/pipeline"
	dbquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
	dbserver "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/server"
	keyval "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
	metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
	dbsvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
	utils "github.com/deven96/ahnlich/sdk/ahnlich-client-go/tests"
)

// Helper to start the DB process for tests
func startDB(t *testing.T) *utils.AhnlichProcess {
	return utils.RunAhnlich(
		t,
		&utils.BinaryFlag{BinaryType: "ahnlich-db"},
	)
}

// Helper to dial the DB gRPC server
func dialDB(t *testing.T, addr string) (*grpc.ClientConn, context.CancelFunc) {
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	conn, err := grpc.DialContext(ctx, addr, grpc.WithInsecure(), grpc.WithBlock())
	require.NoError(t, err)
	return conn, cancel
}

// Shared test data
var (
	storeNoPred = &dbquery.CreateStore{
		Store:         "Diretnan Station",
		Dimension:     5,
		ErrorIfExists: true,
	}
	storeWithPred = &dbquery.CreateStore{
		Store:            "Diretnan Predication",
		Dimension:        5,
		ErrorIfExists:    true,
		CreatePredicates: []string{"is_tyrannical", "rank"},
	}
)

func TestCreateStore_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	resp, err := client.CreateStore(context.Background(), storeNoPred)
	require.NoError(t, err)
	require.IsType(t, &dbserver.Unit{}, resp)
}

func TestCreateStoreWithPredicates_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	resp, err := client.CreateStore(context.Background(), storeWithPred)
	require.NoError(t, err)
	require.IsType(t, &dbserver.Unit{}, resp)
}

func TestListStores_FindsCreatedStore(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)
	resp, err := client.ListStores(context.Background(), &dbquery.ListStores{})
	require.NoError(t, err)
	require.NotEmpty(t, resp.Stores)
	found := false
	for _, s := range resp.Stores {
		if s.Name == storeNoPred.Store {
			found = true
		}
	}
	require.True(t, found)
}

func TestSetInStore_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "sorcerer"}},
				},
			},
		},
		{
			Key: &keyval.StoreKey{Key: []float32{5, 3, 4, 3.9, 4.9}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"rank": {Value: &metadata.MetadataValue_RawString{RawString: "chunin"}},
				},
			},
		},
	}
	setReq := &dbquery.Set{Store: storeNoPred.Store, Inputs: entries}
	resp, err := client.Set(context.Background(), setReq)
	require.NoError(t, err)
	require.EqualValues(t, 2, resp.Upsert.Inserted)
	require.EqualValues(t, 0, resp.Upsert.Updated)
}

func TestSetInStore_SucceedsWithBinary(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 4, 3, 3.9, 4.9}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"image": {Value: &metadata.MetadataValue_Image{Image: []byte{2, 2, 3, 4, 5, 6, 7}}},
				},
			},
		},
	}
	setReq := &dbquery.Set{Store: storeNoPred.Store, Inputs: entries}
	resp, err := client.Set(context.Background(), setReq)
	require.NoError(t, err)
	require.EqualValues(t, 1, resp.Upsert.Inserted)
}

func TestCreatePredicateIndex_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeWithPred)
	predReq := &dbquery.CreatePredIndex{
		Store:      storeWithPred.Store,
		Predicates: []string{"rank"},
	}
	resp, err := client.CreatePredIndex(context.Background(), predReq)
	require.NoError(t, err)
	require.NotNil(t, resp)
}

func TestGetByPredicate_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeWithPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"rank": {Value: &metadata.MetadataValue_RawString{RawString: "jonin"}},
				},
			},
		},
		{
			Key: &keyval.StoreKey{Key: []float32{2, 3, 4, 5, 6}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"rank": {Value: &metadata.MetadataValue_RawString{RawString: "chunin"}},
				},
			},
		},
	}
	_, _ = client.Set(context.Background(), &dbquery.Set{Store: storeWithPred.Store, Inputs: entries})
	_, _ = client.CreatePredIndex(context.Background(), &dbquery.CreatePredIndex{Store: storeWithPred.Store, Predicates: []string{"rank"}})
	getPredReq := &dbquery.GetPred{
		Store: storeWithPred.Store,
		Condition: &predicates.PredicateCondition{
			Kind: &predicates.PredicateCondition_Value{
				Value: &predicates.Predicate{
					Kind: &predicates.Predicate_Equals{
						Equals: &predicates.Equals{
							Key:   "rank",
							Value: &metadata.MetadataValue{Value: &metadata.MetadataValue_RawString{RawString: "jonin"}},
						},
					},
				},
			},
		},
	}
	resp, err := client.GetPred(context.Background(), getPredReq)
	require.NoError(t, err)
	require.Len(t, resp.Entries, 1)
	require.Equal(t, "jonin", resp.Entries[0].Value.Value["rank"].GetRawString())
}

func TestGetSimN_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeNoPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "ninja"}},
				},
			},
		},
		{
			Key: &keyval.StoreKey{Key: []float32{2, 3, 4, 5, 6}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "samurai"}},
				},
			},
		},
	}
	_, _ = client.Set(context.Background(), &dbquery.Set{Store: storeNoPred.Store, Inputs: entries})
	getSimReq := &dbquery.GetSimN{
		Store:       storeNoPred.Store,
		SearchInput: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
		ClosestN:    1,
		Algorithm:   0, // Use default or set appropriate algorithm enum if required
	}
	resp, err := client.GetSimN(context.Background(), getSimReq)
	require.NoError(t, err)
	require.Len(t, resp.Entries, 1)
	require.Equal(t, []float32{1, 2, 3, 4, 5}, resp.Entries[0].Key.Key)
}

func TestDropPredicateIndex_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeWithPred)
	_, _ = client.CreatePredIndex(context.Background(), &dbquery.CreatePredIndex{Store: storeWithPred.Store, Predicates: []string{"rank"}})
	dropReq := &dbquery.DropPredIndex{
		Store:      storeWithPred.Store,
		Predicates: []string{"rank"},
	}
	resp, err := client.DropPredIndex(context.Background(), dropReq)
	require.NoError(t, err)
	require.NotNil(t, resp)
}

func TestDeletePredicate_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeWithPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"rank": {Value: &metadata.MetadataValue_RawString{RawString: "jonin"}},
				},
			},
		},
	}
	_, _ = client.Set(context.Background(), &dbquery.Set{Store: storeWithPred.Store, Inputs: entries})
	delPredReq := &dbquery.DelPred{
		Store: storeWithPred.Store,
		Condition: &predicates.PredicateCondition{
			Kind: &predicates.PredicateCondition_Value{
				Value: &predicates.Predicate{
					Kind: &predicates.Predicate_Equals{
						Equals: &predicates.Equals{
							Key:   "rank",
							Value: &metadata.MetadataValue{Value: &metadata.MetadataValue_RawString{RawString: "jonin"}},
						},
					},
				},
			},
		},
	}
	resp, err := client.DelPred(context.Background(), delPredReq)
	require.NoError(t, err)
	require.NotNil(t, resp)
}

func TestDeleteKey_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeNoPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "ninja"}},
				},
			},
		},
	}
	_, _ = client.Set(context.Background(), &dbquery.Set{Store: storeNoPred.Store, Inputs: entries})
	delKeyReq := &dbquery.DelKey{
		Store: storeNoPred.Store,
		Keys: []*keyval.StoreKey{
			{Key: []float32{1, 2, 3, 4, 5}},
		},
	}
	resp, err := client.DelKey(context.Background(), delKeyReq)
	require.NoError(t, err)
	require.NotNil(t, resp)
}

func TestDropStore_Succeeds(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeNoPred)
	dropReq := &dbquery.DropStore{Store: storeNoPred.Store}
	resp, err := client.DropStore(context.Background(), dropReq)
	require.NoError(t, err)
	require.NotNil(t, resp)
}

func TestListStores_ReflectsDroppedStore(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeNoPred)
	_, _ = client.CreateStore(context.Background(), storeWithPred)
	_, _ = client.DropStore(context.Background(), &dbquery.DropStore{Store: storeNoPred.Store})
	resp, err := client.ListStores(context.Background(), &dbquery.ListStores{})
	require.NoError(t, err)
	for _, s := range resp.Stores {
		require.NotEqual(t, storeNoPred.Store, s.Name)
	}
}

func TestCreateAndDropKDTreeNonLinearIndex(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)

	// Create KDTree index
	createResp, err := client.CreateNonLinearAlgorithmIndex(context.Background(), &dbquery.CreateNonLinearAlgorithmIndex{
		Store: storeNoPred.Store,
		NonLinearIndices: []*nonlinear.NonLinearIndex{
			{Index: &nonlinear.NonLinearIndex_Kdtree{Kdtree: &nonlinear.KDTreeConfig{}}},
		},
	})
	require.NoError(t, err)
	require.NotNil(t, createResp)

	// Drop KDTree index
	dropResp, err := client.DropNonLinearAlgorithmIndex(context.Background(), &dbquery.DropNonLinearAlgorithmIndex{
		Store:            storeNoPred.Store,
		NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
		ErrorIfNotExists: true,
	})
	require.NoError(t, err)
	require.NotNil(t, dropResp)
}

func TestCreateAndDropHNSWNonLinearIndex(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)

	// Create HNSW index with default config
	createResp, err := client.CreateNonLinearAlgorithmIndex(context.Background(), &dbquery.CreateNonLinearAlgorithmIndex{
		Store: storeNoPred.Store,
		NonLinearIndices: []*nonlinear.NonLinearIndex{
			{Index: &nonlinear.NonLinearIndex_Hnsw{Hnsw: &nonlinear.HNSWConfig{}}},
		},
	})
	require.NoError(t, err)
	require.NotNil(t, createResp)

	// Drop HNSW index
	dropResp, err := client.DropNonLinearAlgorithmIndex(context.Background(), &dbquery.DropNonLinearAlgorithmIndex{
		Store:            storeNoPred.Store,
		NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_HNSW},
		ErrorIfNotExists: true,
	})
	require.NoError(t, err)
	require.NotNil(t, dropResp)
}

func TestPipeline_BulkSetAndGet(t *testing.T) {
	t.Parallel()
	proc := startDB(t)
	defer proc.Kill()
	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()
	client := dbsvc.NewDBServiceClient(conn)
	_, _ = client.CreateStore(context.Background(), storeNoPred)
	entries := []*keyval.DbStoreEntry{
		{
			Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4, 5}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "ninja"}},
				},
			},
		},
		{
			Key: &keyval.StoreKey{Key: []float32{2, 3, 4, 5, 6}},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"job": {Value: &metadata.MetadataValue_RawString{RawString: "samurai"}},
				},
			},
		},
	}
	setQ := &pipeline.DBQuery{Query: &pipeline.DBQuery_Set{Set: &dbquery.Set{Store: storeNoPred.Store, Inputs: entries}}}
	getQ := &pipeline.DBQuery{Query: &pipeline.DBQuery_GetKey{GetKey: &dbquery.GetKey{Store: storeNoPred.Store, Keys: []*keyval.StoreKey{{Key: []float32{1, 2, 3, 4, 5}}}}}}
	pipelineReq := &pipeline.DBRequestPipeline{Queries: []*pipeline.DBQuery{setQ, getQ}}
	resp, err := client.Pipeline(context.Background(), pipelineReq)
	require.NoError(t, err)
	require.Len(t, resp.Responses, 2)
	setResp := resp.Responses[0].GetSet()
	getResp := resp.Responses[1].GetGet()
	require.NotNil(t, setResp)
	require.NotNil(t, getResp)
	require.Len(t, getResp.Entries, 1)
	require.Equal(t, []float32{1, 2, 3, 4, 5}, getResp.Entries[0].Key.Key)
}
