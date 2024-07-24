package client

import (
	"fmt"

	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"
)

type AhnlichDBResponse interface{}

func ParseDBResponse(serverResult *dbResponse.ServerResult) ([]AhnlichDBResponse, error) {
	serverResponse := make([]AhnlichDBResponse, 0)
	for _, result := range serverResult.Results {
		switch result := result.(type) {
		case *dbResponse.Result__Err:
			return nil, fmt.Errorf("error response from server: %v", result)
		case *dbResponse.Result__Ok:
			response := result.Value
			switch response := response.(type) {
			case *dbResponse.ServerResponse__InfoServer:
				serverResponse = append(serverResponse, response.Value)
			case *dbResponse.ServerResponse__Pong:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__GetSimN:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__ClientList:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__Get:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__Set:
				serverResponse = append(serverResponse, response.Value)
			case *dbResponse.ServerResponse__Del:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__StoreList:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__Unit:
				serverResponse = append(serverResponse, *response)
			case *dbResponse.ServerResponse__CreateIndex:
				serverResponse = append(serverResponse, *response)
			default:
				return nil, fmt.Errorf("unknown response type: %T", response)
			}
		default:
			return nil, fmt.Errorf("unknown response type: %T", result)
		}
	}
	return serverResponse, nil
}

// Create a DB response array type from a slice of float32
func MakeDBResponseArrayType(data []float32, v uint8) dbResponse.Array {
	data32 := make([]float32, len(data))
	for i, d := range data {
		data32[i] = float32(d)
	}
	dimensions := struct{ Field0 uint64 }{Field0: uint64(len(data))}
	return dbResponse.Array{
		V:    v,
		Dim:  dimensions,
		Data: data32,
	}
}

func MakeDBResponseMetaDataType(data map[string]string) map[string]dbResponse.MetadataValue {
	metadata := make(map[string]dbResponse.MetadataValue)
	for k, v := range data {
		val := dbResponse.MetadataValue__RawString(v)
		metadata[k] = &val
	}
	return metadata
}
