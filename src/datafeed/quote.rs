use crate::datafeed::types::{QuoteData, QuoteMessage};

pub fn parse_message(json: &str) -> Option<QuoteMessage> {
    serde_json::from_str::<QuoteMessage>(json).ok()
}

pub fn process(json: &str) -> Option<QuoteData> {
    parse_message(json).and_then(|msg| msg.quotes.into_iter().next())
}

pub fn to_json_string(json: &str) -> Option<String> {
    process(json)
        .map(|quote| serde_json::to_string_pretty(&quote).ok())
        .flatten()
}
