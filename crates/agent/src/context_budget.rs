//! Codex-inspired bounds for model-visible context injections.
//!
//! Rules adapted from Codex's model-visible context discipline:
//! - every injected fragment has a hard size cap
//! - prefer middle truncation so start/end cues survive
//! - keep individual retained facts small so compaction summaries stay useful

/// Approximate tokens from UTF-8 bytes (Codex-style 4 bytes/token heuristic).
pub fn approx_token_count(text: &str) -> usize {
    text.len().div_ceil(4)
}

fn approx_bytes_for_tokens(tokens: usize) -> usize {
    tokens.saturating_mul(4)
}

/// Hard caps for common model-visible injections (token units).
pub const MAX_COMPACTION_SUMMARY_TOKENS: usize = 8_000;
pub const MAX_SPAWN_AGENT_MESSAGE_TOKENS: usize = 6_000;
pub const MAX_SPAWN_AGENT_OUTPUT_TOKENS: usize = 4_000;
pub const MAX_RETAINED_FACT_TOKENS: usize = 400;
pub const MAX_RETAINED_FACTS: usize = 32;

const TRUNCATION_MARKER: &str = "\n…[truncated to fit context budget]…\n";

/// Truncate from the middle when over budget, keeping head and tail.
pub fn truncate_middle_to_tokens(text: &str, max_tokens: usize) -> String {
    let max_bytes = approx_bytes_for_tokens(max_tokens);
    if text.len() <= max_bytes {
        return text.to_string();
    }
    if max_bytes == 0 {
        return String::new();
    }

    let marker = TRUNCATION_MARKER;
    if max_bytes <= marker.len() {
        return marker.chars().take(max_bytes).collect();
    }

    let keep = max_bytes - marker.len();
    let head_len = keep / 2;
    let tail_len = keep - head_len;

    let mut head_end = head_len.min(text.len());
    while head_end > 0 && !text.is_char_boundary(head_end) {
        head_end -= 1;
    }
    let mut tail_start = text.len().saturating_sub(tail_len);
    while tail_start < text.len() && !text.is_char_boundary(tail_start) {
        tail_start += 1;
    }
    if tail_start < head_end {
        return text[..head_end].to_string();
    }

    let mut out = String::with_capacity(max_bytes);
    out.push_str(&text[..head_end]);
    out.push_str(marker);
    out.push_str(&text[tail_start..]);
    out
}

/// Cap a list of retained facts: drop extras and truncate each item.
pub fn cap_retained_facts(facts: impl IntoIterator<Item = String>) -> Vec<String> {
    facts
        .into_iter()
        .take(MAX_RETAINED_FACTS)
        .map(|fact| truncate_middle_to_tokens(&fact, MAX_RETAINED_FACT_TOKENS))
        .filter(|fact| !fact.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_middle_keeps_ends() {
        let text = format!(
            "{}MIDDLE{}{}",
            "A".repeat(200),
            "B".repeat(200),
            "C".repeat(200)
        );
        let truncated = truncate_middle_to_tokens(&text, 50);
        assert!(truncated.starts_with('A'));
        assert!(truncated.ends_with('C'));
        assert!(truncated.contains("truncated"));
        assert!(approx_token_count(&truncated) <= 55);
    }

    #[test]
    fn cap_retained_facts_limits_count_and_size() {
        let facts = (0..40)
            .map(|i| format!("fact-{i}-{}", "x".repeat(2_000)))
            .collect::<Vec<_>>();
        let capped = cap_retained_facts(facts);
        assert_eq!(capped.len(), MAX_RETAINED_FACTS);
        assert!(
            capped
                .iter()
                .all(|f| approx_token_count(f) <= MAX_RETAINED_FACT_TOKENS + 5)
        );
    }
}
