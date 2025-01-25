mod server_test;

use ahnlich_types::keyval::StoreKey;
use std::collections::HashMap;

pub fn word_to_vector() -> HashMap<String, StoreKey> {
    let words = std::fs::read_to_string("src/tests/mock_data.json").unwrap();

    serde_json::from_str(&words).unwrap()
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
