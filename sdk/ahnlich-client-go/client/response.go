package client

import (
	"fmt"

	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"
)

func ParseDBResponse(serverResult *dbResponse.ServerResult) ([]dbResponse.ServerResponse, error) {
	serverResponse := make([]dbResponse.ServerResponse, 0)
	for _, result := range serverResult.Results {
		switch result := result.(type) {
		case *dbResponse.Result__Err:
			return nil, fmt.Errorf("error response from server: %v", result)
		case *dbResponse.Result__Ok:
			response := result.Value
			switch response := response.(type) {
			case *dbResponse.ServerResponse__InfoServer:
				// response.Value
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__Pong:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__GetSimN:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__ClientList:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__Get:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__Set:
				serverResponse = append(serverResponse, response)
				// response.Value
			case *dbResponse.ServerResponse__Del:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__StoreList:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__Unit:
				serverResponse = append(serverResponse, response)
			case *dbResponse.ServerResponse__CreateIndex:
				serverResponse = append(serverResponse, response)
			default:
				return nil, fmt.Errorf("unknown response type: %T", response)
			}
		default:
			return nil, fmt.Errorf("unknown response type: %T", result)
		}
	}
	return serverResponse, nil
}
