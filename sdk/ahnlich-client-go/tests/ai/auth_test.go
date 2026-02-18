package ai_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
	aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
	utils "github.com/deven96/ahnlich/sdk/ahnlich-client-go/tests"
)

func dialAIWithTLS(t *testing.T, addr string, cfg *utils.AuthConfig) (*grpc.ClientConn, context.CancelFunc) {
	creds := utils.ClientTLSCredentials(t, cfg.CertPEM)
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	conn, err := grpc.DialContext(ctx, addr, grpc.WithTransportCredentials(creds), grpc.WithBlock())
	require.NoError(t, err)
	return conn, cancel
}

func authCtxAI(username, apiKey string) context.Context {
	md := metadata.Pairs("authorization", "Bearer "+username+":"+apiKey)
	return metadata.NewOutgoingContext(context.Background(), md)
}

func aiGRPCCode(err error) codes.Code {
	return status.Code(err)
}

func TestAI_UnauthenticatedRequestRejected(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"aiuser": "aipassword"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-ai"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialAIWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := aisvc.NewAIServiceClient(conn)
	_, err := client.Ping(context.Background(), &aiquery.Ping{})
	require.Error(t, err)
	require.Equal(t, codes.Unauthenticated, aiGRPCCode(err))
}

func TestAI_WrongCredentialsRejected(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"aiuser": "aipassword"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-ai"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialAIWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := aisvc.NewAIServiceClient(conn)
	_, err := client.Ping(authCtxAI("aiuser", "wrongpassword"), &aiquery.Ping{})
	require.Error(t, err)
	require.Equal(t, codes.Unauthenticated, aiGRPCCode(err))
}

func TestAI_ValidCredentialsAccepted(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"aiuser": "aipassword"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-ai"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialAIWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := aisvc.NewAIServiceClient(conn)
	_, err := client.Ping(authCtxAI("aiuser", "aipassword"), &aiquery.Ping{})
	require.NoError(t, err)
}

func TestAI_NoAuthServerAcceptsRequestsWithoutToken(t *testing.T) {
	proc := utils.RunAhnlich(t, &utils.BinaryFlag{BinaryType: "ahnlich-ai"})
	defer proc.Kill()

	conn, cancel := dialAI(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()

	client := aisvc.NewAIServiceClient(conn)
	_, err := client.Ping(context.Background(), &aiquery.Ping{})
	require.NoError(t, err)
}

func TestAI_WithAuthAndDBWithoutAuth(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"aiuser": "aipassword"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-ai"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialAIWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := aisvc.NewAIServiceClient(conn)

	_, err := client.Ping(authCtxAI("aiuser", "aipassword"), &aiquery.Ping{})
	require.NoError(t, err)

	_, err = client.ListStores(authCtxAI("aiuser", "aipassword"), &aiquery.ListStores{})
	require.NoError(t, err)
}
