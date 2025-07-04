package ai_test


// ai_client_test.go
package ahnlichgotest_test         // keep tests out-of-package to avoid import cycles

import (
	"context"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
	aimodel "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
	aipipeline "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/pipeline"
	aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
	aiserver "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/server"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
	aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"

	utils "github.com/deven96/ahnlich/sdk/ahnlich-client-go/tests/ahnlichgotest"
)

func startAI(t *testing.T) *utils.AhnlichProcess {
	return utils.RunAhnlich(
		t,
		&utils.BinaryOption{BinaryType: "ai"}, // <-- key difference vs. DB
	)
}

func dialAI(t *testing.T, addr string) *grpc.ClientConn {
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	t.Cleanup(cancel)
	conn, err := grpc.DialContext(ctx, addr, grpc.WithInsecure(), grpc.WithBlock())
	require.NoError(t, err)
	return conn
}

/* -------------------------------------------------------------------------
   shared test data
   -------------------------------------------------------------------------*/
var (
	storeNoPred = &aiquery.CreateStore{
		Store:         "Diretnan Stores",
		QueryModel:    aimodel.AiModel_ALL_MINI_LM_L6_V2,
		IndexModel:    aimodel.AiModel_ALL_MINI_LM_L6_V2,
		ErrorIfExists: true,
		StoreOriginal: true,
	}
	storeWithPred = &aiquery.CreateStore{
		Store:         "Diretnan Predication Stores",
		QueryModel:    aimodel.AiModel_ALL_MINI_LM_L6_V2,
		IndexModel:    aimodel.AiModel_ALL_MINI_LM_L6_V2,
		Predicates:    []string{"special", "brand"},
		ErrorIfExists: true,
		StoreOriginal: true,
	}
)

/* -------------------------------------------------------------------------
   tests (one-to-one with your Python suite)
   -------------------------------------------------------------------------*/

func TestCreateStoreOK(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()

	client := aisvc.NewAiServiceClient(conn)
	_, err := client.CreateStore(context.Background(), storeNoPred)
	require.NoError(t, err)
}

func TestCreateStoreAlreadyExists(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)
	_, err := client.CreateStore(context.Background(), storeNoPred)

	st, ok := status.FromError(err)
	require.True(t, ok)
	require.Equal(t, codes.AlreadyExists, st.Code())
}

func TestGetPredicates(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeWithPred)

	entries := []*keyval.AiStoreEntry{
		{
			Key: &keyval.StoreInput{RawString: "Jordan One"},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"brand": {RawString: "Nike"},
				},
			},
		},
		{
			Key: &keyval.StoreInput{RawString: "Yeezey"},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"brand": {RawString: "Adidas"},
				},
			},
		},
	}
	_, _ = client.Set(context.Background(), &aiquery.Set{
		Store:            storeWithPred.Store,
		Inputs:           entries,
		PreprocessAction: preprocess.PreprocessAction_NoPreprocessing,
	})

	cond := &predicates.PredicateCondition{
		Value: &predicates.Predicate{
			Equals: &predicates.Equals{
				Key:   "brand",
				Value: &metadata.MetadataValue{RawString: "Nike"},
			},
		},
	}
	resp, err := client.GetPred(context.Background(), &aiquery.GetPred{
		Store:     storeWithPred.Store,
		Condition: cond,
	})
	require.NoError(t, err)
	require.Len(t, resp.Entries, 1)
	require.Equal(t, "Jordan One", resp.Entries[0].Key.RawString)
}

func TestCreateAndDropPredIndex(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)

	cre, _ := client.CreatePredIndex(context.Background(), &aiquery.CreatePredIndex{
		Store:      storeNoPred.Store,
		Predicates: []string{"super_sales"},
	})
	require.EqualValues(t, 1, cre.CreatedIndexes)

	drop, _ := client.DropPredIndex(context.Background(), &aiquery.DropPredIndex{
		Store:            storeNoPred.Store,
		Predicates:       []string{"super_sales"},
		ErrorIfNotExists: true,
	})
	require.EqualValues(t, 1, drop.DeletedCount)

	_, err := client.DropPredIndex(context.Background(), &aiquery.DropPredIndex{
		Store:            storeNoPred.Store,
		Predicates:       []string{"nonexistent"},
		ErrorIfNotExists: true,
	})
	st, _ := status.FromError(err)
	require.Equal(t, codes.NotFound, st.Code())
}

func TestDeleteAndGetKey(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeWithPred)

	_, _ = client.Set(context.Background(), &aiquery.Set{
		Store: storeWithPred.Store,
		Inputs: []*keyval.AiStoreEntry{{
			Key: &keyval.StoreInput{RawString: "Yeezey"},
			Value: &keyval.StoreValue{
				Value: map[string]*metadata.MetadataValue{
					"brand": {RawString: "Adidas"},
				},
			},
		}},
	})

	del, _ := client.DelKey(context.Background(), &aiquery.DelKey{
		Store: storeWithPred.Store,
		Keys:  []*keyval.StoreInput{{RawString: "Yeezey"}},
	})
	require.EqualValues(t, 1, del.DeletedCount)
}

func TestDropStoreAndPurge(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	_, _ = client.CreateStore(context.Background(), storeNoPred)
	_, _ = client.CreateStore(context.Background(), storeWithPred)

	drop, _ := client.DropStore(context.Background(), &aiquery.DropStore{
		Store:            storeWithPred.Store,
		ErrorIfNotExists: true,
	})
	require.EqualValues(t, 1, drop.DeletedCount)

	purge, _ := client.PurgeStores(context.Background(), &aiquery.PurgeStores{})
	require.GreaterOrEqual(t, purge.DeletedCount, int64(1))
}

func TestListClients(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	resp, err := client.ListClients(context.Background(), &aiquery.ListClients{})
	require.NoError(t, err)
	require.NotZero(t, len(resp.Clients))
}

func TestPipelineSuccess(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	req := &aipipeline.AiRequestPipeline{
		Queries: []*aipipeline.AiQuery{
			{CreateStore: storeWithPred},
			{
				Set: &aiquery.Set{
					Store: storeWithPred.Store,
					Inputs: []*keyval.AiStoreEntry{{
						Key: &keyval.StoreInput{RawString: "Product1"},
						Value: &keyval.StoreValue{
							Value: map[string]*metadata.MetadataValue{
								"category": {RawString: "Electronics"},
							},
						},
					}},
					PreprocessAction: preprocess.PreprocessAction_NoPreprocessing,
				},
			},
			{
				CreatePredIndex: &aiquery.CreatePredIndex{
					Store:      storeWithPred.Store,
					Predicates: []string{"category"},
				},
			},
		},
	}

	resp, err := client.Pipeline(context.Background(), req)
	require.NoError(t, err)
	require.Len(t, resp.Responses, 3)
	require.NotNil(t, resp.Responses[0].GetUnit())
	require.NotNil(t, resp.Responses[1].GetSet())
	require.NotNil(t, resp.Responses[2].GetCreateIndex())
}

func TestPipelineError(t *testing.T) {
	proc := startAI(t)
	conn := dialAI(t, proc.ServerAddr)
	defer conn.Close()
	client := aisvc.NewAiServiceClient(conn)

	bad := &aipipeline.AiRequestPipeline{
		Queries: []*aipipeline.AiQuery{{
			Set: &aiquery.Set{
				Store: "does_not_exist",
				Inputs: []*keyval.AiStoreEntry{{
					Key:   &keyval.StoreInput{RawString: "P1"},
					Value: &keyval.StoreValue{},
				}},
			},
		}},
	}

	resp, err := client.Pipeline(context.Background(), bad)
	require.NoError(t, err)
	require.Len(t, resp.Responses, 1)
	require.NotNil(t, resp.Responses[0].GetError())
	require.Contains(t, strings.ToLower(resp.Responses[0].GetError().Message), "not found")
}

func TestMain(m *testing.M) {
	os.Exit(m.Run())
}