use crate::hnsw::Node;
use std::collections::HashMap;

pub const SEACH_TEXT: &'static str =
    "Football fans enjoy gathering to watch matches at sports bars.";

pub const MOST_SIMILAR: [&'static str; 3] = [
    "Attending football games at the stadium is an exciting experience.",
    "On sunny days, people often gather outdoors for a friendly game of football.",
    "Rainy weather can sometimes lead to canceled outdoor events like football matches.",
];

pub fn word_to_vector() -> HashMap<String, Node> {
    let words = std::fs::read_to_string("src/tests/fixtures/mock_data.json").unwrap();

    let json: HashMap<String, Vec<f32>> = serde_json::from_str(&words).unwrap();

    json.into_iter()
        .map(|(key, value)| (key, Node::new(value)))
        .collect()
}
