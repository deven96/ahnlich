module github.com/deven96/ahnlich/examples/go/audio-search

go 1.23.0

require (
	github.com/deven96/ahnlich/sdk/ahnlich-client-go v0.0.0
	google.golang.org/grpc v1.73.0
)

require (
	golang.org/x/net v0.38.0 // indirect
	golang.org/x/sys v0.31.0 // indirect
	golang.org/x/text v0.23.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250324211829-b45e905df463 // indirect
	google.golang.org/protobuf v1.36.6 // indirect
)

replace github.com/deven96/ahnlich/sdk/ahnlich-client-go => ../../../sdk/ahnlich-client-go
