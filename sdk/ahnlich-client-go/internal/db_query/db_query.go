package internal_db_query


import (
	"fmt"
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/serde"
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/bincode"
)


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

type Array struct {
	V uint8
	Dim struct {Field0 uint64}
	Data []float32
}

func (obj *Array) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serializer.SerializeU8(obj.V); err != nil { return err }
	if err := serialize_tuple1_u64(obj.Dim, serializer); err != nil { return err }
	if err := serialize_vector_f32(obj.Data, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Array) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeArray(deserializer serde.Deserializer) (Array, error) {
	var obj Array
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeU8(); err == nil { obj.V = val } else { return obj, err }
	if val, err := deserialize_tuple1_u64(deserializer); err == nil { obj.Dim = val } else { return obj, err }
	if val, err := deserialize_vector_f32(deserializer); err == nil { obj.Data = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeArray(input []byte) (Array, error) {
	if input == nil {
		var obj Array
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeArray(deserializer)
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

type Query interface {
	isQuery()
	Serialize(serializer serde.Serializer) error
	BincodeSerialize() ([]byte, error)
}

func DeserializeQuery(deserializer serde.Deserializer) (Query, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil { return nil, err }

	switch index {
	case 0:
		if val, err := load_Query__CreateStore(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Query__GetKey(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_Query__GetPred(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_Query__GetSimN(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_Query__CreatePredIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_Query__CreateNonLinearAlgorithmIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_Query__DropPredIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_Query__DropNonLinearAlgorithmIndex(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_Query__Set(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_Query__DelKey(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 10:
		if val, err := load_Query__DelPred(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 11:
		if val, err := load_Query__DropStore(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 12:
		if val, err := load_Query__InfoServer(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 13:
		if val, err := load_Query__ListStores(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 14:
		if val, err := load_Query__ListClients(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 15:
		if val, err := load_Query__Ping(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Query: %d", index)
	}
}

func BincodeDeserializeQuery(input []byte) (Query, error) {
	if input == nil {
		var obj Query
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeQuery(deserializer)
	if err == nil && deserializer.GetBufferOffset() < uint64(len(input)) {
		return obj, fmt.Errorf("Some input bytes were not read")
	}
	return obj, err
}

type Query__CreateStore struct {
	Store string
	Dimension uint64
	CreatePredicates []string
	NonLinearIndices []NonLinearAlgorithm
	ErrorIfExists bool
}

func (*Query__CreateStore) isQuery() {}

func (obj *Query__CreateStore) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serializer.SerializeU64(obj.Dimension); err != nil { return err }
	if err := serialize_vector_str(obj.CreatePredicates, serializer); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__CreateStore) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__CreateStore(deserializer serde.Deserializer) (Query__CreateStore, error) {
	var obj Query__CreateStore
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Dimension = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.CreatePredicates = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__GetKey struct {
	Store string
	Keys []Array
}

func (*Query__GetKey) isQuery() {}

func (obj *Query__GetKey) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_Array(obj.Keys, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__GetKey) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__GetKey(deserializer serde.Deserializer) (Query__GetKey, error) {
	var obj Query__GetKey
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_Array(deserializer); err == nil { obj.Keys = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__GetPred struct {
	Store string
	Condition PredicateCondition
}

func (*Query__GetPred) isQuery() {}

func (obj *Query__GetPred) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(2)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.Condition.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__GetPred) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__GetPred(deserializer serde.Deserializer) (Query__GetPred, error) {
	var obj Query__GetPred
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Condition = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__GetSimN struct {
	Store string
	SearchInput Array
	ClosestN uint64
	Algorithm Algorithm
	Condition *PredicateCondition
}

func (*Query__GetSimN) isQuery() {}

func (obj *Query__GetSimN) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(3)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.SearchInput.Serialize(serializer); err != nil { return err }
	if err := serializer.SerializeU64(obj.ClosestN); err != nil { return err }
	if err := obj.Algorithm.Serialize(serializer); err != nil { return err }
	if err := serialize_option_PredicateCondition(obj.Condition, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__GetSimN) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__GetSimN(deserializer serde.Deserializer) (Query__GetSimN, error) {
	var obj Query__GetSimN
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializeArray(deserializer); err == nil { obj.SearchInput = val } else { return obj, err }
	if val, err := deserializer.DeserializeU64(); err == nil { obj.ClosestN = val } else { return obj, err }
	if val, err := DeserializeAlgorithm(deserializer); err == nil { obj.Algorithm = val } else { return obj, err }
	if val, err := deserialize_option_PredicateCondition(deserializer); err == nil { obj.Condition = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__CreatePredIndex struct {
	Store string
	Predicates []string
}

func (*Query__CreatePredIndex) isQuery() {}

func (obj *Query__CreatePredIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(4)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_str(obj.Predicates, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__CreatePredIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__CreatePredIndex(deserializer serde.Deserializer) (Query__CreatePredIndex, error) {
	var obj Query__CreatePredIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.Predicates = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__CreateNonLinearAlgorithmIndex struct {
	Store string
	NonLinearIndices []NonLinearAlgorithm
}

func (*Query__CreateNonLinearAlgorithmIndex) isQuery() {}

func (obj *Query__CreateNonLinearAlgorithmIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(5)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__CreateNonLinearAlgorithmIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__CreateNonLinearAlgorithmIndex(deserializer serde.Deserializer) (Query__CreateNonLinearAlgorithmIndex, error) {
	var obj Query__CreateNonLinearAlgorithmIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__DropPredIndex struct {
	Store string
	Predicates []string
	ErrorIfNotExists bool
}

func (*Query__DropPredIndex) isQuery() {}

func (obj *Query__DropPredIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(6)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_str(obj.Predicates, serializer); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__DropPredIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__DropPredIndex(deserializer serde.Deserializer) (Query__DropPredIndex, error) {
	var obj Query__DropPredIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_str(deserializer); err == nil { obj.Predicates = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__DropNonLinearAlgorithmIndex struct {
	Store string
	NonLinearIndices []NonLinearAlgorithm
	ErrorIfNotExists bool
}

func (*Query__DropNonLinearAlgorithmIndex) isQuery() {}

func (obj *Query__DropNonLinearAlgorithmIndex) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(7)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_NonLinearAlgorithm(obj.NonLinearIndices, serializer); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__DropNonLinearAlgorithmIndex) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__DropNonLinearAlgorithmIndex(deserializer serde.Deserializer) (Query__DropNonLinearAlgorithmIndex, error) {
	var obj Query__DropNonLinearAlgorithmIndex
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_NonLinearAlgorithm(deserializer); err == nil { obj.NonLinearIndices = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__Set struct {
	Store string
	Inputs []struct {Field0 Array; Field1 map[string]MetadataValue}
}

func (*Query__Set) isQuery() {}

func (obj *Query__Set) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(8)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_tuple2_Array_map_str_to_MetadataValue(obj.Inputs, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__Set) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__Set(deserializer serde.Deserializer) (Query__Set, error) {
	var obj Query__Set
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_tuple2_Array_map_str_to_MetadataValue(deserializer); err == nil { obj.Inputs = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__DelKey struct {
	Store string
	Keys []Array
}

func (*Query__DelKey) isQuery() {}

func (obj *Query__DelKey) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(9)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serialize_vector_Array(obj.Keys, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__DelKey) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__DelKey(deserializer serde.Deserializer) (Query__DelKey, error) {
	var obj Query__DelKey
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserialize_vector_Array(deserializer); err == nil { obj.Keys = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__DelPred struct {
	Store string
	Condition PredicateCondition
}

func (*Query__DelPred) isQuery() {}

func (obj *Query__DelPred) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(10)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := obj.Condition.Serialize(serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__DelPred) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__DelPred(deserializer serde.Deserializer) (Query__DelPred, error) {
	var obj Query__DelPred
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := DeserializePredicateCondition(deserializer); err == nil { obj.Condition = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__DropStore struct {
	Store string
	ErrorIfNotExists bool
}

func (*Query__DropStore) isQuery() {}

func (obj *Query__DropStore) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(11)
	if err := serializer.SerializeStr(obj.Store); err != nil { return err }
	if err := serializer.SerializeBool(obj.ErrorIfNotExists); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__DropStore) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__DropStore(deserializer serde.Deserializer) (Query__DropStore, error) {
	var obj Query__DropStore
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserializer.DeserializeStr(); err == nil { obj.Store = val } else { return obj, err }
	if val, err := deserializer.DeserializeBool(); err == nil { obj.ErrorIfNotExists = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__InfoServer struct {
}

func (*Query__InfoServer) isQuery() {}

func (obj *Query__InfoServer) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(12)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__InfoServer) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__InfoServer(deserializer serde.Deserializer) (Query__InfoServer, error) {
	var obj Query__InfoServer
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__ListStores struct {
}

func (*Query__ListStores) isQuery() {}

func (obj *Query__ListStores) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(13)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__ListStores) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__ListStores(deserializer serde.Deserializer) (Query__ListStores, error) {
	var obj Query__ListStores
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__ListClients struct {
}

func (*Query__ListClients) isQuery() {}

func (obj *Query__ListClients) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(14)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__ListClients) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__ListClients(deserializer serde.Deserializer) (Query__ListClients, error) {
	var obj Query__ListClients
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Query__Ping struct {
}

func (*Query__Ping) isQuery() {}

func (obj *Query__Ping) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	serializer.SerializeVariantIndex(15)
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *Query__Ping) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func load_Query__Ping(deserializer serde.Deserializer) (Query__Ping, error) {
	var obj Query__Ping
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ServerQuery struct {
	Queries []Query
	TraceId *string
}

func (obj *ServerQuery) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil { return err }
	if err := serialize_vector_Query(obj.Queries, serializer); err != nil { return err }
	if err := serialize_option_str(obj.TraceId, serializer); err != nil { return err }
	serializer.DecreaseContainerDepth()
	return nil
}

func (obj *ServerQuery) BincodeSerialize() ([]byte, error) {
	if obj == nil {
		return nil, fmt.Errorf("Cannot serialize null object")
	}
	serializer := bincode.NewSerializer();
	if err := obj.Serialize(serializer); err != nil { return nil, err }
	return serializer.GetBytes(), nil
}

func DeserializeServerQuery(deserializer serde.Deserializer) (ServerQuery, error) {
	var obj ServerQuery
	if err := deserializer.IncreaseContainerDepth(); err != nil { return obj, err }
	if val, err := deserialize_vector_Query(deserializer); err == nil { obj.Queries = val } else { return obj, err }
	if val, err := deserialize_option_str(deserializer); err == nil { obj.TraceId = val } else { return obj, err }
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

func BincodeDeserializeServerQuery(input []byte) (ServerQuery, error) {
	if input == nil {
		var obj ServerQuery
		return obj, fmt.Errorf("Cannot deserialize null array")
	}
	deserializer := bincode.NewDeserializer(input);
	obj, err := DeserializeServerQuery(deserializer)
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

func serialize_tuple1_u64(value struct {Field0 uint64}, serializer serde.Serializer) error {
	if err := serializer.SerializeU64(value.Field0); err != nil { return err }
	return nil
}

func deserialize_tuple1_u64(deserializer serde.Deserializer) (struct {Field0 uint64}, error) {
	var obj struct {Field0 uint64}
	if val, err := deserializer.DeserializeU64(); err == nil { obj.Field0 = val } else { return obj, err }
	return obj, nil
}

func serialize_tuple2_Array_map_str_to_MetadataValue(value struct {Field0 Array; Field1 map[string]MetadataValue}, serializer serde.Serializer) error {
	if err := value.Field0.Serialize(serializer); err != nil { return err }
	if err := serialize_map_str_to_MetadataValue(value.Field1, serializer); err != nil { return err }
	return nil
}

func deserialize_tuple2_Array_map_str_to_MetadataValue(deserializer serde.Deserializer) (struct {Field0 Array; Field1 map[string]MetadataValue}, error) {
	var obj struct {Field0 Array; Field1 map[string]MetadataValue}
	if val, err := DeserializeArray(deserializer); err == nil { obj.Field0 = val } else { return obj, err }
	if val, err := deserialize_map_str_to_MetadataValue(deserializer); err == nil { obj.Field1 = val } else { return obj, err }
	return obj, nil
}

func serialize_vector_Array(value []Array, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_Array(deserializer serde.Deserializer) ([]Array, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]Array, length)
	for i := range(obj) {
		if val, err := DeserializeArray(deserializer); err == nil { obj[i] = val } else { return nil, err }
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

func serialize_vector_Query(value []Query, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := item.Serialize(serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_Query(deserializer serde.Deserializer) ([]Query, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]Query, length)
	for i := range(obj) {
		if val, err := DeserializeQuery(deserializer); err == nil { obj[i] = val } else { return nil, err }
	}
	return obj, nil
}

func serialize_vector_f32(value []float32, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serializer.SerializeF32(item); err != nil { return err }
	}
	return nil
}

func deserialize_vector_f32(deserializer serde.Deserializer) ([]float32, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]float32, length)
	for i := range(obj) {
		if val, err := deserializer.DeserializeF32(); err == nil { obj[i] = val } else { return nil, err }
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

func serialize_vector_tuple2_Array_map_str_to_MetadataValue(value []struct {Field0 Array; Field1 map[string]MetadataValue}, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil { return err }
	for _, item := range(value) {
		if err := serialize_tuple2_Array_map_str_to_MetadataValue(item, serializer); err != nil { return err }
	}
	return nil
}

func deserialize_vector_tuple2_Array_map_str_to_MetadataValue(deserializer serde.Deserializer) ([]struct {Field0 Array; Field1 map[string]MetadataValue}, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil { return nil, err }
	obj := make([]struct {Field0 Array; Field1 map[string]MetadataValue}, length)
	for i := range(obj) {
		if val, err := deserialize_tuple2_Array_map_str_to_MetadataValue(deserializer); err == nil { obj[i] = val } else { return nil, err }
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

