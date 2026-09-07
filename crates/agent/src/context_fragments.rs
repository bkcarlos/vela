//! Model-visible context injections with hard size caps and stable markers.
//!
//! Every host-owned injection that lands in a LanguageModel request should go
//! through a [`ContextualFragment`] so truncation and marker conventions stay
//! consistent (Codex-inspired dual-layer context: messages + retained facts).

use crate::context_budget::{
    MAX_COMPACTION_SUMMARY_TOKENS, MAX_RETAINED_FACT_TOKENS, MAX_SPAWN_AGENT_OUTPUT_TOKENS,
    truncate_middle_to_tokens,
};

/// A typed, budgeted fragment injected into model-visible context.
pub trait ContextualFragment {
    fn kind(&self) -> &'static str;
    /// LanguageModel role string: `"user"`, `"developer"`, or `"system"`.
    fn role(&self) -> &'static str;
    /// `(start, end)` markers wrapping the body. Empty markers skip wrapping.
    fn markers() -> (&'static str, &'static str)
    where
        Self: Sized;
    fn body(&self) -> String;
    fn max_tokens(&self) -> usize;

    fn render(&self) -> String
    where
        Self: Sized,
    {
        let truncated = truncate_middle_to_tokens(&self.body(), self.max_tokens());
        let (start, end) = Self::markers();
        if start.is_empty() && end.is_empty() {
            truncated
        } else {
            format!("{start}\n{truncated}\n{end}")
        }
    }

    fn matches_text(text: &str) -> bool
    where
        Self: Sized,
    {
        let (start, end) = Self::markers();
        !start.is_empty() && !end.is_empty() && text.contains(start) && text.contains(end)
    }
}

/// Compaction summary inserted after auto/manual compaction.
pub struct CompactionSummaryFragment {
    pub summary: String,
}

impl ContextualFragment for CompactionSummaryFragment {
    fn kind(&self) -> &'static str {
        "compaction_summary"
    }

    fn role(&self) -> &'static str {
        "user"
    }

    fn markers() -> (&'static str, &'static str) {
        ("<<<COMPACTION_SUMMARY>>>", "<<<END_COMPACTION_SUMMARY>>>")
    }

    fn body(&self) -> String {
        format!(
            "The previous conversation was compacted. Use this summary as context:\n\n{}",
            self.summary
        )
    }

    fn max_tokens(&self) -> usize {
        MAX_COMPACTION_SUMMARY_TOKENS
    }
}

/// Host-owned facts that survive message-window compaction.
pub struct RetainedFactsFragment {
    pub facts: Vec<String>,
}

impl ContextualFragment for RetainedFactsFragment {
    fn kind(&self) -> &'static str {
        "retained_facts"
    }

    fn role(&self) -> &'static str {
        "user"
    }

    fn markers() -> (&'static str, &'static str) {
        ("<<<RETAINED_FACTS>>>", "<<<END_RETAINED_FACTS>>>")
    }

    fn body(&self) -> String {
        let mut out = String::from(
            "Retained facts from earlier work (host-owned; independent of the message window):\n",
        );
        for fact in &self.facts {
            let capped = truncate_middle_to_tokens(fact, MAX_RETAINED_FACT_TOKENS);
            if !capped.trim().is_empty() {
                out.push_str("- ");
                out.push_str(&capped);
                out.push('\n');
            }
        }
        out
    }

    fn max_tokens(&self) -> usize {
        // Cap the whole block generously while keeping individual facts small.
        MAX_RETAINED_FACT_TOKENS.saturating_mul(crate::context_budget::MAX_RETAINED_FACTS)
    }
}

/// Typed parent-visible notification when a subagent finishes.
pub struct SubagentNotificationFragment {
    pub agent_session_id: String,
    pub status: String,
    pub summary: String,
}

impl ContextualFragment for SubagentNotificationFragment {
    fn kind(&self) -> &'static str {
        "subagent_notification"
    }

    fn role(&self) -> &'static str {
        "user"
    }

    fn markers() -> (&'static str, &'static str) {
        (
            "<<<SUBAGENT_NOTIFICATION>>>",
            "<<<END_SUBAGENT_NOTIFICATION>>>",
        )
    }

    fn body(&self) -> String {
        format!(
            "session_id: {}\nstatus: {}\nsummary:\n{}",
            self.agent_session_id, self.status, self.summary
        )
    }

    fn max_tokens(&self) -> usize {
        MAX_SPAWN_AGENT_OUTPUT_TOKENS
    }
}

/// Collaboration mode the parent agent should follow for `spawn_agent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MultiAgentCollaborationMode {
    /// Only spawn when the user (or task) clearly requires delegation.
    #[default]
    ExplicitRequestOnly,
    /// Spawn parallel subagents when independent work would help.
    Proactive,
}

impl MultiAgentCollaborationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitRequestOnly => "explicit_request_only",
            Self::Proactive => "proactive",
        }
    }

    pub fn guidance(self) -> &'static str {
        match self {
            Self::ExplicitRequestOnly => {
                "Spawn subagents only when the user explicitly asks or the task clearly \
                 requires isolated parallel work you cannot do yourself with one or two tool calls."
            }
            Self::Proactive => {
                "You may proactively spawn parallel subagents when independent research or \
                 disjoint write scopes would materially speed up the task. Prefer reuse of \
                 existing session_id for follow-ups."
            }
        }
    }
}

impl From<settings::MultiAgentMode> for MultiAgentCollaborationMode {
    fn from(mode: settings::MultiAgentMode) -> Self {
        match mode {
            settings::MultiAgentMode::ExplicitRequestOnly => Self::ExplicitRequestOnly,
            settings::MultiAgentMode::Proactive => Self::Proactive,
        }
    }
}

pub struct MultiAgentModeFragment {
    pub mode: MultiAgentCollaborationMode,
}

impl ContextualFragment for MultiAgentModeFragment {
    fn kind(&self) -> &'static str {
        "multi_agent_mode"
    }

    fn role(&self) -> &'static str {
        "user"
    }

    fn markers() -> (&'static str, &'static str) {
        ("<<<MULTI_AGENT_MODE>>>", "<<<END_MULTI_AGENT_MODE>>>")
    }

    fn body(&self) -> String {
        format!(
            "Multi-agent collaboration mode: {}.\n{}",
            self.mode.as_str(),
            self.mode.guidance()
        )
    }

    fn max_tokens(&self) -> usize {
        400
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compaction_summary_renders_with_markers_and_truncation() {
        let fragment = CompactionSummaryFragment {
            summary: format!("{}MIDDLE{}", "A".repeat(20_000), "Z".repeat(20_000)),
        };
        let rendered = fragment.render();
        assert!(CompactionSummaryFragment::matches_text(&rendered));
        assert!(rendered.contains("<<<COMPACTION_SUMMARY>>>"));
        assert!(rendered.starts_with("<<<COMPACTION_SUMMARY>>>"));
        assert!(rendered.contains("The previous conversation was compacted"));
        assert!(rendered.contains('A'));
        assert!(rendered.contains('Z'));
        assert!(rendered.contains("truncated"));
    }

    #[test]
    fn retained_facts_lists_bullets() {
        let fragment = RetainedFactsFragment {
            facts: vec!["alpha".into(), "beta".into()],
        };
        let rendered = fragment.render();
        assert!(RetainedFactsFragment::matches_text(&rendered));
        assert!(rendered.contains("- alpha"));
        assert!(rendered.contains("- beta"));
    }

    #[test]
    fn subagent_notification_includes_session_and_status() {
        let fragment = SubagentNotificationFragment {
            agent_session_id: "sess-1".into(),
            status: "completed".into(),
            summary: "done".into(),
        };
        let rendered = fragment.render();
        assert!(SubagentNotificationFragment::matches_text(&rendered));
        assert!(rendered.contains("session_id: sess-1"));
        assert!(rendered.contains("status: completed"));
        assert!(rendered.contains("done"));
    }

    #[test]
    fn multi_agent_mode_explicit_and_proactive() {
        let explicit = MultiAgentModeFragment {
            mode: MultiAgentCollaborationMode::ExplicitRequestOnly,
        }
        .render();
        assert!(MultiAgentModeFragment::matches_text(&explicit));
        assert!(explicit.contains("explicit_request_only"));

        let proactive = MultiAgentModeFragment {
            mode: MultiAgentCollaborationMode::Proactive,
        }
        .render();
        assert!(proactive.contains("proactive"));
    }

    #[test]
    fn empty_markers_skip_wrap() {
        struct Bare;
        impl ContextualFragment for Bare {
            fn kind(&self) -> &'static str {
                "bare"
            }
            fn role(&self) -> &'static str {
                "user"
            }
            fn markers() -> (&'static str, &'static str) {
                ("", "")
            }
            fn body(&self) -> String {
                "hello".into()
            }
            fn max_tokens(&self) -> usize {
                10
            }
        }
        assert_eq!(Bare.render(), "hello");
        assert!(!Bare::matches_text("hello"));
    }
}
