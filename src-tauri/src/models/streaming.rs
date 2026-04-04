use serde_json::Value;

/// Incrementally extracts complete JSON block objects from a growing text buffer.
/// Tracks how many blocks have already been extracted so it only yields new ones.
pub struct BlockExtractor {
    extracted_count: usize,
}

impl BlockExtractor {
    pub fn new() -> Self {
        BlockExtractor { extracted_count: 0 }
    }

    /// Attempt to extract new complete blocks from the accumulated buffer.
    /// Returns any newly-completed blocks since the last call.
    pub fn extract_new_blocks(&mut self, buffer: &str) -> Vec<Value> {
        let mut new_blocks = Vec::new();

        // Find the "blocks" array opening
        let blocks_start = match find_blocks_array_start(buffer) {
            Some(pos) => pos,
            None => return new_blocks,
        };

        // Extract individual block objects from the array region
        let array_region = &buffer[blocks_start..];
        let all_blocks = extract_complete_objects(array_region);

        // Only return blocks we haven't seen yet
        if all_blocks.len() > self.extracted_count {
            for block in all_blocks.into_iter().skip(self.extracted_count) {
                self.extracted_count += 1;
                new_blocks.push(block);
            }
        }

        new_blocks
    }
}

/// Find the character position right after `"blocks": [`
fn find_blocks_array_start(s: &str) -> Option<usize> {
    // Look for "blocks" followed by : and [
    let idx = s.find("\"blocks\"")?;
    let after_key = &s[idx + 8..];
    // Skip whitespace and colon
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];
    let bracket_pos = after_colon.find('[')?;
    Some(idx + 8 + colon_pos + 1 + bracket_pos + 1)
}

/// Extract all complete JSON objects from a string that starts inside a JSON array.
/// Handles nested braces correctly.
fn extract_complete_objects(s: &str) -> Vec<Value> {
    let mut objects = Vec::new();
    let mut depth = 0;
    let mut obj_start: Option<usize> = None;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match ch {
            '{' => {
                if depth == 0 {
                    obj_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = obj_start {
                        let obj_str = &s[start..=i];
                        if let Ok(val) = serde_json::from_str::<Value>(obj_str) {
                            // Only include if it has a "type" field (it's a block)
                            if val.get("type").is_some() {
                                objects.push(val);
                            }
                        }
                        obj_start = None;
                    }
                }
            }
            ']' if depth == 0 => {
                // End of blocks array
                break;
            }
            _ => {}
        }
    }

    objects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_blocks_incrementally() {
        let mut extractor = BlockExtractor::new();

        let partial = r#"{"confidence": 0.8, "blocks": [{"type": "text", "heading": "Hi", "body": "Hello"}"#;
        let blocks = extractor.extract_new_blocks(partial);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "text");

        // Add another block
        let more = r#"{"confidence": 0.8, "blocks": [{"type": "text", "heading": "Hi", "body": "Hello"}, {"type": "metric", "label": "X", "value": "5", "trend": "up"}"#;
        let blocks = extractor.extract_new_blocks(more);
        assert_eq!(blocks.len(), 1); // Only the new one
        assert_eq!(blocks[0]["type"], "metric");
    }

    #[test]
    fn test_no_blocks_yet() {
        let mut extractor = BlockExtractor::new();
        let partial = r#"{"confidence": 0.8, "bl"#;
        let blocks = extractor.extract_new_blocks(partial);
        assert_eq!(blocks.len(), 0);
    }
}
