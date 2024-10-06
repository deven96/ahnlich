package internal_ai_query


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

type AIQuery interface {
	isAIQuery()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeAIQuery(deserializer serde.Deserializer) (AIQuery, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_AIQuery__CreateStore(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_AIQuery__GetPred(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_AIQuery__GetSimN(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_AIQuery__CreatePredIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_AIQuery__CreateNonLinearAlgorithmIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_AIQuery__DropPredIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_AIQuery__DropNonLinearAlgorithmIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_AIQuery__Set(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_AIQuery__DelKey(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_AIQuery__DropStore(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 10:
		if val, err := load_AIQuery__InfoServer(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 11:
		if val, err := load_AIQuery__ListStores(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 12:
		if val, err := load_AIQuery__PurgeStores(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 13:
		if val, err := load_AIQuery__Ping(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for AIQuery: %d", index)
	}
}

func BincodeDeserializeAIQuery(input []byte) (AIQuery, error) {
	if input == nil {
		var obj AIQuery
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIQuery(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type AIQuery__CreateStore struct {
	Store string
	QueryModel AIModel
	IndexModel AIModel
	Predicates []string
	NonLinearIndices []NonLinearAlgorithm
}

func (*AIQuery__CreateStore) isAIQuery() {}

func (obj *AIQuery__CreateStore) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.QueryModel.Serialize(serializer); err != nil { return err }
	if err := obj.IndexModel.Serialize(serializer); err != nil { return err }
	if err := serialize_vector_str(obj.Predicates, serializer); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__CreateStore) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__CreateStore(deserializer serde.Deserializer) (AIQuery__CreateStore, error) {
	var obj AIQuery__CreateStore
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializeAIModel(deserializer); err == nil { obj.QueryModel = val } else { return obj, err }
	if val, err := DeserializeAIModel(deserializer); err == nil { obj.IndexModel = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.Predicates = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__GetPred struct {
	Store string
	Condition PredicateCondition
}

func (*AIQuery__GetPred) isAIQuery() {}

func (obj *AIQuery__GetPred) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.Condition.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__GetPred) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__GetPred(deserializer serde.Deserializer) (AIQuery__GetPred, error) {
	var obj AIQuery__GetPred
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Condition = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__GetSimN struct {
	Store string
	SearchInput StoreInput
	Condition *PredicateCondition
	ClosestN uint64
	Algorithm Algorithm
}

func (*AIQuery__GetSimN) isAIQuery() {}

func (obj *AIQuery__GetSimN) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.SearchInput.Serialize(serializer); err != nil { return err }
	if err := serialize_option_PredicateCondition(obj.Condition, serializer); err != nil { return err }
	if err := serializer.SerializeU64(obj.ClosestN); err != nil { return err }
	if err := obj.Algorithm.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__GetSimN) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__GetSimN(deserializer serde.Deserializer) (AIQuery__GetSimN, error) {
	var obj AIQuery__GetSimN
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializeStoreInput(deserializer); err == nil { obj.SearchInput = val } else { return obj, err }
	if val, err := deserialize_option_PredicateCondition(deserializer); err == nil { obj.Condition = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.ClosestN = val } else { return obj, err }
	if val, err := DeserializeAlgorithm(deserializer); err == nil { obj.Algorithm = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__CreatePredIndex struct {
	Store string
	Predicates []string
}

func (*AIQuery__CreatePredIndex) isAIQuery() {}

func (obj *AIQuery__CreatePredIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(3)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_str(obj.Predicates, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__CreatePredIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__CreatePredIndex(deserializer serde.Deserializer) (AIQuery__CreatePredIndex, error) {
	var obj AIQuery__CreatePredIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.Predicates = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__CreateNonLinearAlgorithmIndex struct {
	Store string
	NonLinearIndices []NonLinearAlgorithm
}

func (*AIQuery__CreateNonLinearAlgorithmIndex) isAIQuery() {}

func (obj *AIQuery__CreateNonLinearAlgorithmIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(4)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__CreateNonLinearAlgorithmIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__CreateNonLinearAlgorithmIndex(deserializer serde.Deserializer) (AIQuery__CreateNonLinearAlgorithmIndex, error) {
	var obj AIQuery__CreateNonLinearAlgorithmIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__DropPredIndex struct {
	Store string
	Predicates []string
	ErrorIfNotExists bool
}

func (*AIQuery__DropPredIndex) isAIQuery() {}

func (obj *AIQuery__DropPredIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(5)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_str(obj.Predicates, serializer); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__DropPredIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__DropPredIndex(deserializer serde.Deserializer) (AIQuery__DropPredIndex, error) {
	var obj AIQuery__DropPredIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.Predicates = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__DropNonLinearAlgorithmIndex struct {
	Store string
	NonLinearIndices []NonLinearAlgorithm
	ErrorIfNotExists bool
}

func (*AIQuery__DropNonLinearAlgorithmIndex) isAIQuery() {}

func (obj *AIQuery__DropNonLinearAlgorithmIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(6)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__DropNonLinearAlgorithmIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__DropNonLinearAlgorithmIndex(deserializer serde.Deserializer) (AIQuery__DropNonLinearAlgorithmIndex, error) {
	var obj AIQuery__DropNonLinearAlgorithmIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__Set struct {
	Store string
	Inputs []struct {Field0 StoreInput; Field1 map[string]MetadataValue}
	PreprocessAction PreprocessAction
}

func (*AIQuery__Set) isAIQuery() {}

func (obj *AIQuery__Set) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(7)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_tuple2_StoreInput_map_str_to_MetadataValue(obj.Inputs, serializer); err != nil { return err }
	if err := obj.PreprocessAction.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__Set) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__Set(deserializer serde.Deserializer) (AIQuery__Set, error) {
	var obj AIQuery__Set
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_tuple2_StoreInput_map_str_to_MetadataValue(deserializer); err == nil { obj.Inputs = val } else { return obj, err }
	if val, err := DeserializePreprocessAction(deserializer); err == nil { obj.PreprocessAction = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__DelKey struct {
	Store string
	Key StoreInput
}

func (*AIQuery__DelKey) isAIQuery() {}

func (obj *AIQuery__DelKey) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(8)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.Key.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__DelKey) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__DelKey(deserializer serde.Deserializer) (AIQuery__DelKey, error) {
	var obj AIQuery__DelKey
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializeStoreInput(deserializer); err == nil { obj.Key = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__DropStore struct {
	Store string
	ErrorIfNotExists bool
}

func (*AIQuery__DropStore) isAIQuery() {}

func (obj *AIQuery__DropStore) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(9)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__DropStore) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__DropStore(deserializer serde.Deserializer) (AIQuery__DropStore, error) {
	var obj AIQuery__DropStore
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__InfoServer struct {
}

func (*AIQuery__InfoServer) isAIQuery() {}

func (obj *AIQuery__InfoServer) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(10)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__InfoServer) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__InfoServer(deserializer serde.Deserializer) (AIQuery__InfoServer, error) {
	var obj AIQuery__InfoServer
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__ListStores struct {
}

func (*AIQuery__ListStores) isAIQuery() {}

func (obj *AIQuery__ListStores) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(11)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__ListStores) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__ListStores(deserializer serde.Deserializer) (AIQuery__ListStores, error) {
	var obj AIQuery__ListStores
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__PurgeStores struct {
}

func (*AIQuery__PurgeStores) isAIQuery() {}

func (obj *AIQuery__PurgeStores) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(12)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__PurgeStores) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__PurgeStores(deserializer serde.Deserializer) (AIQuery__PurgeStores, error) {
	var obj AIQuery__PurgeStores
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIQuery__Ping struct {
}

func (*AIQuery__Ping) isAIQuery() {}

func (obj *AIQuery__Ping) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(13)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIQuery__Ping) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_AIQuery__Ping(deserializer serde.Deserializer) (AIQuery__Ping, error) {
	var obj AIQuery__Ping
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type AIServerQuery struct {
	Queries []AIQuery
	TraceId *string
}

func (obj *AIServerQuery) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serialize_vector_AIQuery(obj.Queries, serializer); err != nil { return err }
	if err := serialize_option_str(obj.TraceId, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *AIServerQuery) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeAIServerQuery(deserializer serde.Deserializer) (AIServerQuery, error) {
	var obj AIServerQuery
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserialize_vector_AIQuery(deserializer); err == nil { obj.Queries = val } else { return obj, err }
	if val, err := deserialize_option_str(deserializer); err == nil { obj.TraceId = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeAIServerQuery(input []byte) (AIServerQuery, error) {
	if input == nil {
		var obj AIServerQuery
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAIServerQuery(deserializer)
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

type Algorithm interface {
	isAlgorithm()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeAlgorithm(deserializer serde.Deserializer) (Algorithm, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_Algorithm__EuclideanDistance(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Algorithm__DotProductSimilarity(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_Algorithm__CosineSimilarity(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_Algorithm__KdTree(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Algorithm: %d", index)
	}
}

func BincodeDeserializeAlgorithm(input []byte) (Algorithm, error) {
	if input == nil {
		var obj Algorithm
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeAlgorithm(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type Algorithm__EuclideanDistance struct {
}

func (*Algorithm__EuclideanDistance) isAlgorithm() {}

func (obj *Algorithm__EuclideanDistance) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Algorithm__EuclideanDistance) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Algorithm__EuclideanDistance(deserializer serde.Deserializer) (Algorithm__EuclideanDistance, error) {
	var obj Algorithm__EuclideanDistance
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Algorithm__DotProductSimilarity struct {
}

func (*Algorithm__DotProductSimilarity) isAlgorithm() {}

func (obj *Algorithm__DotProductSimilarity) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Algorithm__DotProductSimilarity) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Algorithm__DotProductSimilarity(deserializer serde.Deserializer) (Algorithm__DotProductSimilarity, error) {
	var obj Algorithm__DotProductSimilarity
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Algorithm__CosineSimilarity struct {
}

func (*Algorithm__CosineSimilarity) isAlgorithm() {}

func (obj *Algorithm__CosineSimilarity) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Algorithm__CosineSimilarity) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Algorithm__CosineSimilarity(deserializer serde.Deserializer) (Algorithm__CosineSimilarity, error) {
	var obj Algorithm__CosineSimilarity
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Algorithm__KdTree struct {
}

func (*Algorithm__KdTree) isAlgorithm() {}

func (obj *Algorithm__KdTree) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(3)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Algorithm__KdTree) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Algorithm__KdTree(deserializer serde.Deserializer) (Algorithm__KdTree, error) {
	var obj Algorithm__KdTree
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ImageAction interface {
	isImageAction()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeImageAction(deserializer serde.Deserializer) (ImageAction, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_ImageAction__ResizeImage(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_ImageAction__ErrorIfDimensionsMismatch(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for ImageAction: %d", index)
	}
}

func BincodeDeserializeImageAction(input []byte) (ImageAction, error) {
	if input == nil {
		var obj ImageAction
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeImageAction(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type ImageAction__ResizeImage struct {
}

func (*ImageAction__ResizeImage) isImageAction() {}

func (obj *ImageAction__ResizeImage) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ImageAction__ResizeImage) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_ImageAction__ResizeImage(deserializer serde.Deserializer) (ImageAction__ResizeImage, error) {
	var obj ImageAction__ResizeImage
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ImageAction__ErrorIfDimensionsMismatch struct {
}

func (*ImageAction__ErrorIfDimensionsMismatch) isImageAction() {}

func (obj *ImageAction__ErrorIfDimensionsMismatch) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ImageAction__ErrorIfDimensionsMismatch) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_ImageAction__ErrorIfDimensionsMismatch(deserializer serde.Deserializer) (ImageAction__ErrorIfDimensionsMismatch, error) {
	var obj ImageAction__ErrorIfDimensionsMismatch
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
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

type NonLinearAlgorithm interface {
	isNonLinearAlgorithm()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeNonLinearAlgorithm(deserializer serde.Deserializer) (NonLinearAlgorithm, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_NonLinearAlgorithm__KdTree(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for NonLinearAlgorithm: %d", index)
	}
}

func BincodeDeserializeNonLinearAlgorithm(input []byte) (NonLinearAlgorithm, error) {
	if input == nil {
		var obj NonLinearAlgorithm
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeNonLinearAlgorithm(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type NonLinearAlgorithm__KdTree struct {
}

func (*NonLinearAlgorithm__KdTree) isNonLinearAlgorithm() {}

func (obj *NonLinearAlgorithm__KdTree) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *NonLinearAlgorithm__KdTree) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_NonLinearAlgorithm__KdTree(deserializer serde.Deserializer) (NonLinearAlgorithm__KdTree, error) {
	var obj NonLinearAlgorithm__KdTree
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Predicate interface {
	isPredicate()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializePredicate(deserializer serde.Deserializer) (Predicate, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_Predicate__Equals(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Predicate__NotEquals(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_Predicate__In(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_Predicate__NotIn(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Predicate: %d", index)
	}
}

func BincodeDeserializePredicate(input []byte) (Predicate, error) {
	if input == nil {
		var obj Predicate
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializePredicate(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type Predicate__Equals struct {
	Key string
	Value MetadataValue
}

func (*Predicate__Equals) isPredicate() {}

func (obj *Predicate__Equals) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeStr(obj.Key); err != nil { return err }
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Predicate__Equals) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Predicate__Equals(deserializer serde.Deserializer) (Predicate__Equals, error) {
	var obj Predicate__Equals
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Key = val } else { return obj, err }
	if val, err := DeserializeMetadataValue(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Predicate__NotEquals struct {
	Key string
	Value MetadataValue
}

func (*Predicate__NotEquals) isPredicate() {}

func (obj *Predicate__NotEquals) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeStr(obj.Key); err != nil { return err }
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Predicate__NotEquals) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Predicate__NotEquals(deserializer serde.Deserializer) (Predicate__NotEquals, error) {
	var obj Predicate__NotEquals
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Key = val } else { return obj, err }
	if val, err := DeserializeMetadataValue(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Predicate__In struct {
	Key string
	Value []MetadataValue
}

func (*Predicate__In) isPredicate() {}

func (obj *Predicate__In) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	if err := serializer.SerializeStr(obj.Key); err != nil { return err }
	if err := serialize_vector_MetadataValue(obj.Value, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Predicate__In) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Predicate__In(deserializer serde.Deserializer) (Predicate__In, error) {
	var obj Predicate__In
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Key = val } else { return obj, err }
	if val, err := deserialize_vector_MetadataValue(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Predicate__NotIn struct {
	Key string
	Value []MetadataValue
}

func (*Predicate__NotIn) isPredicate() {}

func (obj *Predicate__NotIn) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(3)
	if err := serializer.SerializeStr(obj.Key); err != nil { return err }
	if err := serialize_vector_MetadataValue(obj.Value, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Predicate__NotIn) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Predicate__NotIn(deserializer serde.Deserializer) (Predicate__NotIn, error) {
	var obj Predicate__NotIn
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Key = val } else { return obj, err }
	if val, err := deserialize_vector_MetadataValue(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type PredicateCondition interface {
	isPredicateCondition()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializePredicateCondition(deserializer serde.Deserializer) (PredicateCondition, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_PredicateCondition__Value(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_PredicateCondition__And(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_PredicateCondition__Or(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for PredicateCondition: %d", index)
	}
}

func BincodeDeserializePredicateCondition(input []byte) (PredicateCondition, error) {
	if input == nil {
		var obj PredicateCondition
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializePredicateCondition(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type PredicateCondition__Value struct {
	Value Predicate
}

func (*PredicateCondition__Value) isPredicateCondition() {}

func (obj *PredicateCondition__Value) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *PredicateCondition__Value) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_PredicateCondition__Value(deserializer serde.Deserializer) (PredicateCondition__Value, error) {
	var obj PredicateCondition__Value
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializePredicate(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type PredicateCondition__And struct {
	Field0 PredicateCondition
	Field1 PredicateCondition
}

func (*PredicateCondition__And) isPredicateCondition() {}

func (obj *PredicateCondition__And) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := obj.Field0.Serialize(serializer); err != nil { return err }
	if err := obj.Field1.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *PredicateCondition__And) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_PredicateCondition__And(deserializer serde.Deserializer) (PredicateCondition__And, error) {
	var obj PredicateCondition__And
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Field0 = val } else { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Field1 = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type PredicateCondition__Or struct {
	Field0 PredicateCondition
	Field1 PredicateCondition
}

func (*PredicateCondition__Or) isPredicateCondition() {}

func (obj *PredicateCondition__Or) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	if err := obj.Field0.Serialize(serializer); err != nil { return err }
	if err := obj.Field1.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *PredicateCondition__Or) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_PredicateCondition__Or(deserializer serde.Deserializer) (PredicateCondition__Or, error) {
	var obj PredicateCondition__Or
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Field0 = val } else { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Field1 = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type PreprocessAction interface {
	isPreprocessAction()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializePreprocessAction(deserializer serde.Deserializer) (PreprocessAction, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_PreprocessAction__RawString(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_PreprocessAction__Image(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for PreprocessAction: %d", index)
	}
}

func BincodeDeserializePreprocessAction(input []byte) (PreprocessAction, error) {
	if input == nil {
		var obj PreprocessAction
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializePreprocessAction(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type PreprocessAction__RawString struct {
	Value StringAction
}

func (*PreprocessAction__RawString) isPreprocessAction() {}

func (obj *PreprocessAction__RawString) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *PreprocessAction__RawString) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_PreprocessAction__RawString(deserializer serde.Deserializer) (PreprocessAction__RawString, error) {
	var obj PreprocessAction__RawString
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializeStringAction(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type PreprocessAction__Image struct {
	Value ImageAction
}

func (*PreprocessAction__Image) isPreprocessAction() {}

func (obj *PreprocessAction__Image) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := obj.Value.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *PreprocessAction__Image) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_PreprocessAction__Image(deserializer serde.Deserializer) (PreprocessAction__Image, error) {
	var obj PreprocessAction__Image
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := DeserializeImageAction(deserializer); err == nil { obj.Value = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
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

type StringAction interface {
	isStringAction()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeStringAction(deserializer serde.Deserializer) (StringAction, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_StringAction__TruncateIfTokensExceed(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_StringAction__ErrorIfTokensExceed(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for StringAction: %d", index)
	}
}

func BincodeDeserializeStringAction(input []byte) (StringAction, error) {
	if input == nil {
		var obj StringAction
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeStringAction(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type StringAction__TruncateIfTokensExceed struct {
}

func (*StringAction__TruncateIfTokensExceed) isStringAction() {}

func (obj *StringAction__TruncateIfTokensExceed) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *StringAction__TruncateIfTokensExceed) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_StringAction__TruncateIfTokensExceed(deserializer serde.Deserializer) (StringAction__TruncateIfTokensExceed, error) {
	var obj StringAction__TruncateIfTokensExceed
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type StringAction__ErrorIfTokensExceed struct {
}

func (*StringAction__ErrorIfTokensExceed) isStringAction() {}

func (obj *StringAction__ErrorIfTokensExceed) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *StringAction__ErrorIfTokensExceed) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_StringAction__ErrorIfTokensExceed(deserializer serde.Deserializer) (StringAction__ErrorIfTokensExceed, error) {
	var obj StringAction__ErrorIfTokensExceed
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
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

func serialize_option_PredicateCondition(value *PredicateCondition, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil { return err }
		if err := (*value).Serialize(serializer); err != nil { return err }
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil { return err }
	}
	return nil
}

func deserialize_option_PredicateCondition(deserializer serde.Deserializer) (*PredicateCondition, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil { return nil, err }
	if tag {
		value := new(PredicateCondition)
		if val, err := DeserializePredicateCondition(deserializer); err == nil { *value = val } else { return nil, err }
	        return value, nil
	} else {
		return nil, nil
	}
}

func serialize_option_str(value *string, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil { return err }
		if err := serializer.SerializeStr((*value)); err != nil { return err }
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil { return err }
	}
	return nil
}

func deserialize_option_str(deserializer serde.Deserializer) (*string, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil { return nil, err }
	if tag {
		value := new(string)
		if val, err := deserializer.DeserializeStr(); err == nil { *value = val } else { return nil, err }
	        return value, nil
	} else {
		return nil, nil
	}
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

func serialize_vector_AIQuery(value []AIQuery, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_AIQuery(deserializer serde.Deserializer) ([]AIQuery, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]AIQuery, length)
	for i := range(obj) {
		if val, err := DeserializeAIQuery(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_MetadataValue(value []MetadataValue, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_MetadataValue(deserializer serde.Deserializer) ([]MetadataValue, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]MetadataValue, length)
	for i := range(obj) {
		if val, err := DeserializeMetadataValue(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_NonLinearAlgorithm(value []NonLinearAlgorithm, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_NonLinearAlgorithm(deserializer serde.Deserializer) ([]NonLinearAlgorithm, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]NonLinearAlgorithm, length)
	for i := range(obj) {
		if val, err := DeserializeNonLinearAlgorithm(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_str(value []string, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serializer.SerializeStr(item); err != nil { return err }
	}
	return nil
}

func deserialize_vector_str(deserializer serde.Deserializer) ([]string, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]string, length)
	for i := range(obj) {
		if val, err := deserializer.DeserializeStr(); err == nil { obj[i] = val } else { return nil, err }
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

