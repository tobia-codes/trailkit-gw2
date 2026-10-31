//! Splitting raw API array responses into compact NDJSON lines without
//! interpreting the objects (lossless archive, data-pipeline D4, D33).

use anyhow::{Context, Result};
use serde_json::value::RawValue;

use crate::compact_json::compact;

/// Splits a JSON array body into one compact line per element.
///
/// # Errors
/// Returns an error if `body` is not a JSON array.
pub fn array_to_lines(body: &str) -> Result<Vec<String>> {
    let elements: Vec<&RawValue> = serde_json::from_str(body).context("expected a JSON array")?;
    Ok(elements
        .into_iter()
        .map(|element| compact(element.get()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_line_per_element_with_original_key_order() -> Result<()> {
        let lines = array_to_lines(include_str!("../fixtures/prices-sample.json"))?;
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with(r#"{"id":19721,"whitelisted":true,"buys":{"quantity":"#));
        Ok(())
    }

    #[test]
    fn rejects_non_arrays() {
        assert!(array_to_lines(r#"{"id":1}"#).is_err());
    }
}
