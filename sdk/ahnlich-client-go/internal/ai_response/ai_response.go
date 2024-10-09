package internal_ai_response


import (
	"fmt"
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/serde"
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/bincode"
)


type AIModel interface {
	isAIModel()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeAIModel(deserializer serde.Deserializer) (AIModel, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_AIModel__Dalle3(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_AIModel__Llama3(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for AIModel: %d", index)
	}
}

func BincodeDeserializeAIModel(input []byte) (AIModel, error) {
	if input == nil {
		var obj AIModel
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIModel(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIModel__Dalle3 struct {
}

func (*AIModel__Dalle3) isAIModel() {}

func (obj *AIModel__Dalle3) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIModel__Dalle3) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIModel__Dalle3(deserializer serde.Deserializer) (AIModel__Dalle3, error) {
	var obj AIModel__Dalle3
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIModel__Llama3 struct {
}

func (*AIModel__Llama3) isAIModel() {}

func (obj *AIModel__Llama3) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIModel__Llama3) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIModel__Llama3(deserializer serde.Deserializer) (AIModel__Llama3, error) {
	var obj AIModel__Llama3
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerResponse interface {
	isAIServerResponse()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeAIServerResponse(deserializer serde.Deserializer) (AIServerResponse, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_AIServerResponse__Unit(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_AIServerResponse__Pong(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_AIServerResponse__ClientList(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_AIServerResponse__StoreList(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_AIServerResponse__InfoServer(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_AIServerResponse__Set(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_AIServerResponse__Get(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_AIServerResponse__GetSimN(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_AIServerResponse__Del(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_AIServerResponse__CreateIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for AIServerResponse: %d", index)
	}
}

func BincodeDeserializeAIServerResponse(input []byte) (AIServerResponse, error) {
	if input == nil {
		var obj AIServerResponse
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIServerResponse(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIServerResponse__Unit struct {
}

func (*AIServerResponse__Unit) isAIServerResponse() {}

func (obj *AIServerResponse__Unit) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__Unit) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__Unit(deserializer serde.Deserializer) (AIServerResponse__Unit, error) {
	var obj AIServerResponse__Unit
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerResponse__Pong struct {
}

func (*AIServerResponse__Pong) isAIServerResponse() {}

func (obj *AIServerResponse__Pong) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__Pong) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__Pong(deserializer serde.Deserializer) (AIServerResponse__Pong, error) {
	var obj AIServerResponse__Pong
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerResponse__ClientList []ConnectedClient

func (*AIServerResponse__ClientList) isAIServerResponse() {}

func (obj *AIServerResponse__ClientList) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	if err := serialize_vector_ConnectedClient((([]ConnectedClient)(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__ClientList) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__ClientList(deserializer serde.Deserializer) (AIServerResponse__ClientList, error) {
	var obj []ConnectedClient
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__ClientList)(obj), err }
	if val, err := deserialize_vector_ConnectedClient(deserializer); err == nil { obj = val } else { return ((AIServerResponse__ClientList)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__ClientList)(obj), nil
}

type AIServerResponse__StoreList []AIStoreInfo

func (*AIServerResponse__StoreList) isAIServerResponse() {}

func (obj *AIServerResponse__StoreList) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(3)
	if err := serialize_vector_AIStoreInfo((([]AIStoreInfo)(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__StoreList) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__StoreList(deserializer serde.Deserializer) (AIServerResponse__StoreList, error) {
	var obj []AIStoreInfo
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__StoreList)(obj), err }
	if val, err := deserialize_vector_AIStoreInfo(deserializer); err == nil { obj = val } else { return ((AIServerResponse__StoreList)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__StoreList)(obj), nil
}

type AIServerResponse__InfoServer struct {
	Value ServerInfo
}

func (*AIServerResponse__InfoServer) isAIServerResponse() {}

func (obj *AIServerResponse__InfoServer) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(4)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__InfoServer) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__InfoServer(deserializer serde.Deserializer) (AIServerResponse__InfoServer, error) {
	var obj AIServerResponse__InfoServer
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializeServerInfo(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerResponse__Set struct {
	Value StoreUpsert
}

func (*AIServerResponse__Set) isAIServerResponse() {}

func (obj *AIServerResponse__Set) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(5)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__Set) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__Set(deserializer serde.Deserializer) (AIServerResponse__Set, error) {
	var obj AIServerResponse__Set
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializeStoreUpsert(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerResponse__Get []struct {Field0 StoreInput; Field1 map[string]MetadataValue}

func (*AIServerResponse__Get) isAIServerResponse() {}

func (obj *AIServerResponse__Get) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(6)
	if err := serialize_vector_tuple2_StoreInput_map_str_to_MetadataValue((([]struct {Field0 StoreInput; Field1 map[string]MetadataValue})(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__Get) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__Get(deserializer serde.Deserializer) (AIServerResponse__Get, error) {
	var obj []struct {Field0 StoreInput; Field1 map[string]MetadataValue}
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__Get)(obj), err }
	if val, err := deserialize_vector_tuple2_StoreInput_map_str_to_MetadataValue(deserializer); err == nil { obj = val } else { return ((AIServerResponse__Get)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__Get)(obj), nil
}

type AIServerResponse__GetSimN []struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}

func (*AIServerResponse__GetSimN) isAIServerResponse() {}

func (obj *AIServerResponse__GetSimN) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(7)
	if err := serialize_vector_tuple3_StoreInput_map_str_to_MetadataValue_Similarity((([]struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity})(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__GetSimN) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__GetSimN(deserializer serde.Deserializer) (AIServerResponse__GetSimN, error) {
	var obj []struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__GetSimN)(obj), err }
	if val, err := deserialize_vector_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(deserializer); err == nil { obj = val } else { return ((AIServerResponse__GetSimN)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__GetSimN)(obj), nil
}

type AIServerResponse__Del uint64

func (*AIServerResponse__Del) isAIServerResponse() {}

func (obj *AIServerResponse__Del) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(8)
	if err := serializer.SerializeU64(((uint64)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__Del) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__Del(deserializer serde.Deserializer) (AIServerResponse__Del, error) {
	var obj uint64
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__Del)(obj), err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj = val } else { return ((AIServerResponse__Del)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__Del)(obj), nil
}

type AIServerResponse__CreateIndex uint64

func (*AIServerResponse__CreateIndex) isAIServerResponse() {}

func (obj *AIServerResponse__CreateIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(9)
	if err := serializer.SerializeU64(((uint64)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResponse__CreateIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIServerResponse__CreateIndex(deserializer serde.Deserializer) (AIServerResponse__CreateIndex, error) {
	var obj uint64
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (AIServerResponse__CreateIndex)(obj), err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj = val } else { return ((AIServerResponse__CreateIndex)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (AIServerResponse__CreateIndex)(obj), nil
}

type AIServerResult struct {
	Results []Result
}

func (obj *AIServerResult) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serialize_vector_Result(obj.Results, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerResult) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeAIServerResult(deserializer serde.Deserializer) (AIServerResult, error) {
	var obj AIServerResult
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserialize_vector_Result(deserializer); err == nil { obj.Results = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeAIServerResult(input []byte) (AIServerResult, error) {
	if input == nil {
		var obj AIServerResult
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIServerResult(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIStoreInfo struct {
	Name string
	QueryModel AIModel
	IndexModel AIModel
	EmbeddingSize uint64
}

func (obj *AIStoreInfo) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeStr(obj.Name); err != nil { return err }
	if err := obj.QueryModel.Serialize(serializer); err != nil { return err }
	if err := obj.IndexModel.Serialize(serializer); err != nil { return err }
	if err := serializer.SerializeU64(obj.EmbeddingSize); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIStoreInfo) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeAIStoreInfo(deserializer serde.Deserializer) (AIStoreInfo, error) {
	var obj AIStoreInfo
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Name = val } else { return obj, err }
	if val, err := DeserializeAIModel(deserializer); err == nil { obj.QueryModel = val } else { return obj, err }
	if val, err := DeserializeAIModel(deserializer); err == nil { obj.IndexModel = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.EmbeddingSize = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeAIStoreInfo(input []byte) (AIStoreInfo, error) {
	if input == nil {
		var obj AIStoreInfo
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIStoreInfo(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIStoreInputType interface {
	isAIStoreInputType()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeAIStoreInputType(deserializer serde.Deserializer) (AIStoreInputType, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_AIStoreInputType__RawString(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_AIStoreInputType__Image(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for AIStoreInputType: %d", index)
	}
}

func BincodeDeserializeAIStoreInputType(input []byte) (AIStoreInputType, error) {
	if input == nil {
		var obj AIStoreInputType
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIStoreInputType(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIStoreInputType__RawString struct {
}

func (*AIStoreInputType__RawString) isAIStoreInputType() {}

func (obj *AIStoreInputType__RawString) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIStoreInputType__RawString) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIStoreInputType__RawString(deserializer serde.Deserializer) (AIStoreInputType__RawString, error) {
	var obj AIStoreInputType__RawString
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIStoreInputType__Image struct {
}

func (*AIStoreInputType__Image) isAIStoreInputType() {}

func (obj *AIStoreInputType__Image) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIStoreInputType__Image) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIStoreInputType__Image(deserializer serde.Deserializer) (AIStoreInputType__Image, error) {
	var obj AIStoreInputType__Image
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ConnectedClient struct {
	Address string
	TimeConnected SystemTime
}

func (obj *ConnectedClient) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeStr(obj.Address); err != nil { return err }
	if err := obj.TimeConnected.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ConnectedClient) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeConnectedClient(deserializer serde.Deserializer) (ConnectedClient, error) {
	var obj ConnectedClient
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Address = val } else { return obj, err }
	if val, err := DeserializeSystemTime(deserializer); err == nil { obj.TimeConnected = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeConnectedClient(input []byte) (ConnectedClient, error) {
	if input == nil {
		var obj ConnectedClient
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeConnectedClient(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type MetadataValue interface {
	isMetadataValue()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeMetadataValue(deserializer serde.Deserializer) (MetadataValue, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_MetadataValue__RawString(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_MetadataValue__Image(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for MetadataValue: %d", index)
	}
}

func BincodeDeserializeMetadataValue(input []byte) (MetadataValue, error) {
	if input == nil {
		var obj MetadataValue
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeMetadataValue(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type MetadataValue__RawString string

func (*MetadataValue__RawString) isMetadataValue() {}

func (obj *MetadataValue__RawString) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *MetadataValue__RawString) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_MetadataValue__RawString(deserializer serde.Deserializer) (MetadataValue__RawString, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (MetadataValue__RawString)(obj), err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj = val } else { return ((MetadataValue__RawString)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (MetadataValue__RawString)(obj), nil
}

type MetadataValue__Image []uint8

func (*MetadataValue__Image) isMetadataValue() {}

func (obj *MetadataValue__Image) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serialize_vector_u8((([]uint8)(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *MetadataValue__Image) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_MetadataValue__Image(deserializer serde.Deserializer) (MetadataValue__Image, error) {
	var obj []uint8
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (MetadataValue__Image)(obj), err }
	if val, err := deserialize_vector_u8(deserializer); err == nil { obj = val } else { return ((MetadataValue__Image)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (MetadataValue__Image)(obj), nil
}

type Result interface {
	isResult()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeResult(deserializer serde.Deserializer) (Result, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_Result__Ok(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Result__Err(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Result: %d", index)
	}
}

func BincodeDeserializeResult(input []byte) (Result, error) {
	if input == nil {
		var obj Result
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeResult(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type Result__Ok struct {
	Value AIServerResponse
}

func (*Result__Ok) isResult() {}

func (obj *Result__Ok) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Result__Ok) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Result__Ok(deserializer serde.Deserializer) (Result__Ok, error) {
	var obj Result__Ok
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializeAIServerResponse(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Result__Err string

func (*Result__Err) isResult() {}

func (obj *Result__Err) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Result__Err) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Result__Err(deserializer serde.Deserializer) (Result__Err, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (Result__Err)(obj), err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj = val } else { return ((Result__Err)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (Result__Err)(obj), nil
}

type ServerInfo struct {
	Address string
	Version Version
	Type ServerType
	Limit uint64
	Remaining uint64
}

func (obj *ServerInfo) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeStr(obj.Address); err != nil { return err }
	if err := obj.Version.Serialize(serializer); err != nil { return err }
	if err := obj.Type.Serialize(serializer); err != nil { return err }
	if err := serializer.SerializeU64(obj.Limit); err != nil { return err }
	if err := serializer.SerializeU64(obj.Remaining); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ServerInfo) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeServerInfo(deserializer serde.Deserializer) (ServerInfo, error) {
	var obj ServerInfo
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Address = val } else { return obj, err }
	if val, err := DeserializeVersion(deserializer); err == nil { obj.Version = val } else { return obj, err }
	if val, err := DeserializeServerType(deserializer); err == nil { obj.Type = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Limit = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Remaining = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeServerInfo(input []byte) (ServerInfo, error) {
	if input == nil {
		var obj ServerInfo
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeServerInfo(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type ServerType interface {
	isServerType()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeServerType(deserializer serde.Deserializer) (ServerType, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_ServerType__Database(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_ServerType__Ai(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for ServerType: %d", index)
	}
}

func BincodeDeserializeServerType(input []byte) (ServerType, error) {
	if input == nil {
		var obj ServerType
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeServerType(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type ServerType__Database struct {
}

func (*ServerType__Database) isServerType() {}

func (obj *ServerType__Database) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ServerType__Database) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_ServerType__Database(deserializer serde.Deserializer) (ServerType__Database, error) {
	var obj ServerType__Database
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ServerType__Ai struct {
}

func (*ServerType__Ai) isServerType() {}

func (obj *ServerType__Ai) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ServerType__Ai) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_ServerType__Ai(deserializer serde.Deserializer) (ServerType__Ai, error) {
	var obj ServerType__Ai
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Similarity float32

func (obj *Similarity) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeF32(((float32)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Similarity) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeSimilarity(deserializer serde.Deserializer) (Similarity, error) {
	var obj float32
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (Similarity)(obj), err }
	if val, err := deserializer.DeserializeF32(); err == nil { obj = val } else { return ((Similarity)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (Similarity)(obj), nil
}

func BincodeDeserializeSimilarity(input []byte) (Similarity, error) {
	if input == nil {
		var obj Similarity
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeSimilarity(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type StoreInput interface {
	isStoreInput()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeStoreInput(deserializer serde.Deserializer) (StoreInput, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_StoreInput__RawString(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_StoreInput__Image(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for StoreInput: %d", index)
	}
}

func BincodeDeserializeStoreInput(input []byte) (StoreInput, error) {
	if input == nil {
		var obj StoreInput
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeStoreInput(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type StoreInput__RawString string

func (*StoreInput__RawString) isStoreInput() {}

func (obj *StoreInput__RawString) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *StoreInput__RawString) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_StoreInput__RawString(deserializer serde.Deserializer) (StoreInput__RawString, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (StoreInput__RawString)(obj), err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj = val } else { return ((StoreInput__RawString)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (StoreInput__RawString)(obj), nil
}

type StoreInput__Image []uint8

func (*StoreInput__Image) isStoreInput() {}

func (obj *StoreInput__Image) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serialize_vector_u8((([]uint8)(*obj)), serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *StoreInput__Image) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_StoreInput__Image(deserializer serde.Deserializer) (StoreInput__Image, error) {
	var obj []uint8
	if err := deserializer.IncreaseContainerDepth(); err != nil { return (StoreInput__Image)(obj), err }
	if val, err := deserialize_vector_u8(deserializer); err == nil { obj = val } else { return ((StoreInput__Image)(obj)), err }
	deserializer.DecreaseContainerDepth()
	return (StoreInput__Image)(obj), nil
}

type StoreUpsert struct {
	Inserted uint64
	Updated uint64
}

func (obj *StoreUpsert) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeU64(obj.Inserted); err != nil { return err }
	if err := serializer.SerializeU64(obj.Updated); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *StoreUpsert) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeStoreUpsert(deserializer serde.Deserializer) (StoreUpsert, error) {
	var obj StoreUpsert
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Inserted = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Updated = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeStoreUpsert(input []byte) (StoreUpsert, error) {
	if input == nil {
		var obj StoreUpsert
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeStoreUpsert(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type SystemTime struct {
	SecsSinceEpoch uint64
	NanosSinceEpoch uint32
}

func (obj *SystemTime) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeU64(obj.SecsSinceEpoch); err != nil { return err }
	if err := serializer.SerializeU32(obj.NanosSinceEpoch); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *SystemTime) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeSystemTime(deserializer serde.Deserializer) (SystemTime, error) {
	var obj SystemTime
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.SecsSinceEpoch = val } else { return obj, err }
	if val, err := deserializer.DeserializeU32(); err == nil { obj.NanosSinceEpoch = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeSystemTime(input []byte) (SystemTime, error) {
	if input == nil {
		var obj SystemTime
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeSystemTime(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type Version struct {
	Major uint8
	Minor uint16
	Patch uint16
}

func (obj *Version) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeU8(obj.Major); err != nil { return err }
	if err := serializer.SerializeU16(obj.Minor); err != nil { return err }
	if err := serializer.SerializeU16(obj.Patch); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Version) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeVersion(deserializer serde.Deserializer) (Version, error) {
	var obj Version
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeU8(); err == nil { obj.Major = val } else { return obj, err }
	if val, err := deserializer.DeserializeU16(); err == nil { obj.Minor = val } else { return obj, err }
	if val, err := deserializer.DeserializeU16(); err == nil { obj.Patch = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeVersion(input []byte) (Version, error) {
	if input == nil {
		var obj Version
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeVersion(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}
func serialize_map_str_to_MetadataValue(value map[string]MetadataValue, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	offsets := make([]uint64, len(value))
	count := 0
	for k, v := range(value) {
		offsets[count] = serializer.GetBufferOffset()
		count += 1
		if err := serializer.SerializeStr(k); err != nil { return err }
		if err := v.Serialize(serializer); err != nil { return err }
	}
	serializer.SortMapEntries(offsets);
	return nil
}

func deserialize_map_str_to_MetadataValue(deserializer serde.Deserializer) (map[string]MetadataValue, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make(map[string]MetadataValue)
	previous_slice := serde.Slice { 0, 0 }
	for i := 0; i < int(length); i++ {
		var slice serde.Slice
		slice.Start = deserializer.GetBufferOffset()
		var key string
		if val, err := deserializer.DeserializeStr(); err == nil { key = val } else { return nil, err }
		slice.End = deserializer.GetBufferOffset()
		if i > 0 {
			err := deserializer.CheckThatKeySlicesAreIncreasing(previous_slice, slice)
			if err != nil { return nil, err }
		}
		previous_slice = slice
		if val, err := DeserializeMetadataValue(deserializer); err == nil { obj[key] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_tuple2_StoreInput_map_str_to_MetadataValue(value struct {Field0 StoreInput; Field1 map[string]MetadataValue}, serializer serde.Serializer) error {
	if err := value.Field0.Serialize(serializer); err != nil { return err }
	if err := serialize_map_str_to_MetadataValue(value.Field1, serializer); err != nil { return err }
	return nil
}

func deserialize_tuple2_StoreInput_map_str_to_MetadataValue(deserializer serde.Deserializer) (struct {Field0 StoreInput; Field1 map[string]MetadataValue}, error) {
	var obj struct {Field0 StoreInput; Field1 map[string]MetadataValue}
	if val, err := DeserializeStoreInput(deserializer); err == nil { obj.Field0 = val } else { return obj, err }
	if val, err := deserialize_map_str_to_MetadataValue(deserializer); err == nil { obj.Field1 = val } else { return obj, err }
	return obj, nil
}

func serialize_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(value struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}, serializer serde.Serializer) error {
	if err := value.Field0.Serialize(serializer); err != nil { return err }
	if err := serialize_map_str_to_MetadataValue(value.Field1, serializer); err != nil { return err }
	if err := value.Field2.Serialize(serializer); err != nil { return err }
	return nil
}

func deserialize_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(deserializer serde.Deserializer) (struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}, error) {
	var obj struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}
	if val, err := DeserializeStoreInput(deserializer); err == nil { obj.Field0 = val } else { return obj, err }
	if val, err := deserialize_map_str_to_MetadataValue(deserializer); err == nil { obj.Field1 = val } else { return obj, err }
	if val, err := DeserializeSimilarity(deserializer); err == nil { obj.Field2 = val } else { return obj, err }
	return obj, nil
}

func serialize_vector_AIStoreInfo(value []AIStoreInfo, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_AIStoreInfo(deserializer serde.Deserializer) ([]AIStoreInfo, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]AIStoreInfo, length)
	for i := range(obj) {
		if val, err := DeserializeAIStoreInfo(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_ConnectedClient(value []ConnectedClient, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_ConnectedClient(deserializer serde.Deserializer) ([]ConnectedClient, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]ConnectedClient, length)
	for i := range(obj) {
		if val, err := DeserializeConnectedClient(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_Result(value []Result, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_Result(deserializer serde.Deserializer) ([]Result, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]Result, length)
	for i := range(obj) {
		if val, err := DeserializeResult(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_tuple2_StoreInput_map_str_to_MetadataValue(value []struct {Field0 StoreInput; Field1 map[string]MetadataValue}, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serialize_tuple2_StoreInput_map_str_to_MetadataValue(item, serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_tuple2_StoreInput_map_str_to_MetadataValue(deserializer serde.Deserializer) ([]struct {Field0 StoreInput; Field1 map[string]MetadataValue}, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]struct {Field0 StoreInput; Field1 map[string]MetadataValue}, length)
	for i := range(obj) {
		if val, err := deserialize_tuple2_StoreInput_map_str_to_MetadataValue(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(value []struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serialize_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(item, serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(deserializer serde.Deserializer) ([]struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]struct {Field0 StoreInput; Field1 map[string]MetadataValue; Field2 Similarity}, length)
	for i := range(obj) {
		if val, err := deserialize_tuple3_StoreInput_map_str_to_MetadataValue_Similarity(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_u8(value []uint8, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serializer.SerializeU8(item); err != nil { return err }
	}
	return nil
}

func deserialize_vector_u8(deserializer serde.Deserializer) ([]uint8, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]uint8, length)
	for i := range(obj) {
		if val, err := deserializer.DeserializeU8(); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

