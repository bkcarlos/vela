use agent_client_protocol::schema::v1 as acp;
use gpui::{App, SharedString, Task};
use language_model::LanguageModelToolResultContent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{rc::Rc, sync::Arc};
use uuid::Uuid;

use crate::{AgentTool, ThreadEnvironment, ToolCallEventStream, ToolInput};

/// Mark the current work task as completed after all requested work and verification have finished.
/// This archives the task's detailed conversation out of the active model context while retaining
/// the full transcript in the session. Do not call this while work, verification, a user decision,
/// or a requested follow-up remains. Call it exactly once before the final response for a completed
/// work task. Do not use it for casual conversation or informational answers that did not create a
/// work task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteTaskToolInput {
    /// Short title identifying the completed task.
    pub title: String,
    /// What was accomplished and the final result.
    pub outcome: String,
    /// Decisions and constraints that remain relevant to future work.
    #[serde(default)]
    pub retained_context: Vec<String>,
    /// Files changed by the task.
    #[serde(default)]
    pub changed_files: Vec<String>,
    /// Tests, checks, or other verification that completed successfully.
    #[serde(default)]
    pub verification: Vec<String>,
    /// Durable outputs such as commit hashes, release URLs, PRs, tags, or artifact paths.
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletedTaskCheckpoint {
    pub task_id: String,
    pub title: String,
    pub outcome: String,
    pub retained_context: Vec<String>,
    pub changed_files: Vec<String>,
    pub verification: Vec<String>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskToolOutput {
    pub task_completed: CompletedTaskCheckpoint,
}

impl From<CompleteTaskToolOutput> for LanguageModelToolResultContent {
    fn from(output: CompleteTaskToolOutput) -> Self {
        serde_json::to_string(&output)
            .unwrap_or_else(|error| format!("Failed to serialize completed task: {error}"))
            .into()
    }
}

pub struct CompleteTaskTool;

impl AgentTool for CompleteTaskTool {
    type Input = CompleteTaskToolInput;
    type Output = CompleteTaskToolOutput;

    const NAME: &'static str = "complete_task";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        input
            .map(|input| format!("Completed {}", input.title).into())
            .unwrap_or_else(|_| "Complete task".into())
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.spawn(async move |_cx| {
            let input = input.recv().await.map_err(|error| CompleteTaskToolOutput {
                task_completed: CompletedTaskCheckpoint {
                    task_id: "invalid".to_string(),
                    title: "Task completion failed".to_string(),
                    outcome: error.to_string(),
                    retained_context: Vec::new(),
                    changed_files: Vec::new(),
                    verification: Vec::new(),
                    artifacts: Vec::new(),
                },
            })?;
            Ok(CompleteTaskToolOutput {
                task_completed: CompletedTaskCheckpoint {
                    task_id: Uuid::new_v4().to_string(),
                    title: input.title,
                    outcome: input.outcome,
                    retained_context: input.retained_context,
                    changed_files: input.changed_files,
                    verification: input.verification,
                    artifacts: input.artifacts,
                },
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSearchResult {
    pub session_id: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscript {
    pub session_id: String,
    pub start_message: usize,
    pub end_message: usize,
    pub total_messages: usize,
    pub content: String,
    pub truncated: bool,
}

/// Search saved agent sessions by title before using `read_session` to recover details from a
/// completed task. Use this only when the active task checkpoint does not contain enough detail.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSessionsToolInput {
    /// Case-insensitive text to match against session titles. Empty returns recent sessions.
    #[serde(default)]
    pub query: String,
}

pub struct SearchSessionsTool {
    environment: Rc<dyn ThreadEnvironment>,
}

impl SearchSessionsTool {
    pub fn new(environment: Rc<dyn ThreadEnvironment>) -> Self {
        Self { environment }
    }
}

impl AgentTool for SearchSessionsTool {
    type Input = SearchSessionsToolInput;
    type Output = String;

    const NAME: &'static str = "search_sessions";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Search
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        input
            .map(|input| format!("Search sessions for {}", input.query).into())
            .unwrap_or_else(|_| "Search sessions".into())
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let environment = self.environment.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|error| error.to_string())?;
            let task = cx.update(|cx| environment.search_sessions(input.query, cx));
            let sessions = task.await.map_err(|error| error.to_string())?;
            serde_json::to_string(&sessions).map_err(|error| error.to_string())
        })
    }
}

/// Read a bounded range from a saved agent session when a completed task's checkpoint lacks a
/// necessary implementation detail. Prefer the message range recorded in the task checkpoint and
/// request only the smallest useful range.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadSessionToolInput {
    /// Session ID from a task checkpoint or `search_sessions`.
    pub session_id: String,
    /// Zero-based first message to read.
    #[serde(default)]
    pub start_message: usize,
    /// Exclusive zero-based end message. Omit to read through the latest message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_message: Option<usize>,
    /// Maximum characters returned. Defaults to 20,000 and cannot exceed 50,000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<usize>,
}

pub struct ReadSessionTool {
    environment: Rc<dyn ThreadEnvironment>,
}

impl ReadSessionTool {
    pub fn new(environment: Rc<dyn ThreadEnvironment>) -> Self {
        Self { environment }
    }
}

impl AgentTool for ReadSessionTool {
    type Input = ReadSessionToolInput;
    type Output = String;

    const NAME: &'static str = "read_session";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Read
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        input
            .map(|input| format!("Read session {}", input.session_id).into())
            .unwrap_or_else(|_| "Read session".into())
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        _event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let environment = self.environment.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|error| error.to_string())?;
            let maximum_characters = input.max_characters.unwrap_or(20_000).min(50_000);
            let task = cx.update(|cx| {
                environment.read_session(
                    input.session_id,
                    input.start_message,
                    input.end_message,
                    maximum_characters,
                    cx,
                )
            });
            let transcript = task.await.map_err(|error| error.to_string())?;
            serde_json::to_string(&transcript).map_err(|error| error.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    async fn complete_task_returns_structured_checkpoint(cx: &mut TestAppContext) {
        let task = cx.update(|cx| {
            Arc::new(CompleteTaskTool).run(
                ToolInput::resolved(CompleteTaskToolInput {
                    title: "Publish release".to_string(),
                    outcome: "Release published".to_string(),
                    retained_context: vec!["Stable channel".to_string()],
                    changed_files: Vec::new(),
                    verification: vec!["All builds passed".to_string()],
                    artifacts: vec!["v1.2.3".to_string()],
                }),
                ToolCallEventStream::test().0,
                cx,
            )
        });
        let output = task.await.expect("task completion should succeed");
        assert_ne!(output.task_completed.task_id, "invalid");
        assert_eq!(output.task_completed.title, "Publish release");
        assert_eq!(output.task_completed.outcome, "Release published");
    }
}
