use crate::parser::Rule;
use ahnlich_types::keyval::StoreKey;
use ndarray::Array1;
use pest::iterators::Pair;

pub(crate) fn parse_multi_f32_array(f32_arrays_pair: Pair<Rule>) -> Vec<StoreKey> {
    f32_arrays_pair.into_inner().map(parse_f32_array).collect()
}

pub(crate) fn parse_f32_array(pair: Pair<Rule>) -> StoreKey {
    StoreKey(Array1::from_iter(pair.into_inner().map(|f32_pair| {
        f32_pair
            .as_str()
            .parse::<f32>()
            .expect("Cannot parse single f32 num")
    })))
}
