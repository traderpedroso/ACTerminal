use crate::infrastructure::datafeed::types::{QuoteData, QuoteMessage};

pub fn parse_message(json: &str) -> Option<QuoteMessage> {
    match serde_json::from_str::<QuoteMessage>(json) {
        Ok(msg) => Some(msg),
        Err(e) => {
            eprintln!("[QUOTE-PARSE] Error: {}", e);
            None
        }
    }
}

pub fn process(json: &str) -> Option<QuoteData> {
    let msg = parse_message(json)?;

    if msg.quotes.is_empty() {
        eprintln!("[QUOTE-PARSE] No quotes in message!");
        return None;
    }

    msg.quotes.into_iter().next()
}

pub fn to_json_string(json: &str) -> Option<String> {
    process(json).and_then(|quote| serde_json::to_string_pretty(&quote).ok())
}
