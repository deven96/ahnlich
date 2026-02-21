use std::collections::HashMap;

use crate::hnsw::Node;

mod mocked_json_tests;

pub fn word_to_vector() -> HashMap<String, Node> {
    let words = std::fs::read_to_string("src/tests/mock_data.json").unwrap();

    let json: HashMap<String, Vec<f32>> = serde_json::from_str(&words).unwrap();

    json.into_iter()
        .map(|(key, value)| (key, Node::new(value)))
        .collect()
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
