package db_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	dbquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
	dbsvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
	utils "github.com/deven96/ahnlich/sdk/ahnlich-client-go/tests"
)

func dialDBWithTLS(t *testing.T, addr string, cfg *utils.AuthConfig) (*grpc.ClientConn, context.CancelFunc) {
	creds := utils.ClientTLSCredentials(t, cfg.CertPEM)
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	conn, err := grpc.DialContext(ctx, addr, grpc.WithTransportCredentials(creds), grpc.WithBlock())
	require.NoError(t, err)
	return conn, cancel
}

func authCtx(username, apiKey string) context.Context {
	md := metadata.Pairs("authorization", "Bearer "+username+":"+apiKey)
	return metadata.NewOutgoingContext(context.Background(), md)
}

func TestDB_UnauthenticatedRequestRejected(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"alice": "alicepass"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-db"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialDBWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := dbsvc.NewDBServiceClient(conn)
	_, err := client.Ping(context.Background(), &dbquery.Ping{})
	require.Error(t, err)
	require.Equal(t, codes.Unauthenticated, grpcCode(err))
}

func TestDB_WrongCredentialsRejected(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"alice": "alicepass"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-db"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialDBWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := dbsvc.NewDBServiceClient(conn)
	_, err := client.Ping(authCtx("alice", "wrongpassword"), &dbquery.Ping{})
	require.Error(t, err)
	require.Equal(t, codes.Unauthenticated, grpcCode(err))
}

func TestDB_ValidCredentialsAccepted(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{"alice": "alicepass"})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-db"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialDBWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := dbsvc.NewDBServiceClient(conn)
	_, err := client.Ping(authCtx("alice", "alicepass"), &dbquery.Ping{})
	require.NoError(t, err)
}

func TestDB_MultipleUsersAuth(t *testing.T) {
	cfg := utils.GenerateTestTLS(t)
	utils.WriteAuthConfig(t, cfg, map[string]string{
		"alice": "alicepass",
		"bob":   "bobspass1",
	})

	proc := utils.RunAhnlich(t,
		&utils.BinaryFlag{BinaryType: "ahnlich-db"},
		&utils.AuthFlag{Cfg: cfg},
	)
	defer proc.Kill()

	conn, cancel := dialDBWithTLS(t, proc.ServerAddr, cfg)
	defer cancel()
	defer conn.Close()

	client := dbsvc.NewDBServiceClient(conn)

	_, err := client.Ping(authCtx("alice", "alicepass"), &dbquery.Ping{})
	require.NoError(t, err)

	_, err = client.Ping(authCtx("bob", "bobspass1"), &dbquery.Ping{})
	require.NoError(t, err)

	_, err = client.Ping(authCtx("charlie", "charliepass"), &dbquery.Ping{})
	require.Error(t, err)
	require.Equal(t, codes.Unauthenticated, grpcCode(err))
}

func TestDB_NoAuthServerAcceptsRequestsWithoutToken(t *testing.T) {
	proc := utils.RunAhnlich(t, &utils.BinaryFlag{BinaryType: "ahnlich-db"})
	defer proc.Kill()

	conn, cancel := dialDB(t, proc.ServerAddr)
	defer cancel()
	defer conn.Close()

	client := dbsvc.NewDBServiceClient(conn)
	_, err := client.Ping(context.Background(), &dbquery.Ping{})
	require.NoError(t, err)
}

func grpcCode(err error) codes.Code {
	return status.Code(err)
}
