mod server_test;

use std::collections::HashMap;
use types::keyval::StoreKey;

pub fn key_to_words(
    key: &StoreKey,
    vector_to_sentences: &Vec<(StoreKey, String)>,
) -> Option<String> {
    for (vector, word) in vector_to_sentences {
        if key == vector {
            return Some(word.to_string());
        }
    }
    None
}

pub fn word_to_vector() -> HashMap<String, StoreKey> {
    let words = std::fs::read_to_string("src/tests/mock_data.json").unwrap();

    let words_to_vec: HashMap<String, Vec<f32>> = serde_json::from_str(&words).unwrap();

    HashMap::from_iter(
        words_to_vec
            .into_iter()
            .map(|(key, value)| (key, StoreKey(ndarray::Array1::<f32>::from_vec(value)))),
    )
}

pub const SEACH_TEXT: &'static str =
    "Football fans enjoy gathering to watch matches at sports bars.";

pub const MOST_SIMILAR: [&'static str; 3] = [
    "Attending football games at the stadium is an exciting experience.",
    "On sunny days, people often gather outdoors for a friendly game of football.",
    "Rainy weather can sometimes lead to canceled outdoor events like football matches.",
];
pub const SENTENCES: [&'static str; 5] = [
    "On sunny days, people often gather outdoors for a friendly game of football.",
    "Attending football games at the stadium is an exciting experience.",
    "Grilling burgers and hot dogs is a popular activity during summer barbecues.",
    "Rainy weather can sometimes lead to canceled outdoor events like football matches.",
    "Sunny weather is ideal for outdoor activities like playing football.",
];
