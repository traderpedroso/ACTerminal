use crate::datafeed::types::{DomData, DomMessage};

pub fn parse_message(json: &str) -> Option<DomMessage> {
    serde_json::from_str::<DomMessage>(json).ok()
}

pub fn process(json: &str) -> Option<DomData> {
    parse_message(json).and_then(|msg| msg.doms.into_iter().next())
}

pub fn to_json_string(json: &str) -> Option<String> {
    process(json)
        .map(|dom| serde_json::to_string_pretty(&dom).ok())
        .flatten()
}
