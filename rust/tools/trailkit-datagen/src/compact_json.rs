//! Lossless whitespace removal for JSON.
//!
//! The GW2 API returns pretty-printed JSON; NDJSON needs one object per line.
//! Removing the insignificant whitespace keeps every value, number spelling
//! and key order exactly as the API sent it (data-pipeline D33), which a
//! parse + re-serialize round trip would not guarantee.

/// Removes all whitespace outside of string literals from valid JSON text.
#[must_use]
pub fn compact(json: &str) -> String {
    let mut output = String::with_capacity(json.len());
    let mut in_string = false;
    let mut escaped = false;
    for character in json.chars() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            output.push(character);
        } else if !matches!(character, ' ' | '\t' | '\n' | '\r') {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_whitespace_between_tokens() {
        let pretty = "{\n  \"id\": 24,\n  \"buys\": {\n    \"quantity\": 1\n  }\n}";
        assert_eq!(compact(pretty), r#"{"id":24,"buys":{"quantity":1}}"#);
    }

    #[test]
    fn keeps_whitespace_and_escapes_inside_strings() {
        let pretty = r#"{ "name" : "Glob of \"Ecto\" \\ x" , "a b": [ 1 , 2 ] }"#;
        assert_eq!(
            compact(pretty),
            r#"{"name":"Glob of \"Ecto\" \\ x","a b":[1,2]}"#
        );
    }

    #[test]
    fn result_is_equivalent_json() -> Result<(), serde_json::Error> {
        let pretty = include_str!("../fixtures/prices-sample.json");
        let original: serde_json::Value = serde_json::from_str(pretty)?;
        let compacted: serde_json::Value = serde_json::from_str(&compact(pretty))?;
        assert_eq!(original, compacted);
        Ok(())
    }
}
