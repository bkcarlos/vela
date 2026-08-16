use agent_client_protocol::schema::v1 as acp;
use gpui::{App, SharedString, Task};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{AgentTool, ToolCallEventStream, ToolInput};

/// Ask the user one to four questions when requirements, preferences, or implementation choices
/// cannot be inferred safely. Each question offers two to four concrete choices. The UI always
/// provides a free-text answer, a "Chat about this" action, and a Skip action, so do not add
/// "Other", "Chat", "None", or "Skip" as options. When the response action is `chat`, answer the
/// user's message and ask again only if the decision is still needed. Put the recommended choice
/// first and append "(Recommended)" to its label when there is a clear recommendation. Do not use
/// this tool for permission checks or plan approval.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskUserToolInput {
    /// The questions to present together. Provide between one and four questions.
    #[schemars(length(min = 1, max = 4))]
    pub questions: Vec<AskUserToolQuestion>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AskUserToolQuestion {
    /// A complete, specific question ending in a question mark.
    #[schemars(length(min = 1))]
    pub question: String,
    /// A short label displayed with the question. Maximum 12 characters.
    #[schemars(length(min = 1, max = 12))]
    pub header: String,
    /// Two to four concrete choices. Do not include an "Other" or "Skip" choice.
    #[schemars(length(min = 2, max = 4))]
    pub options: Vec<AskUserToolOption>,
    /// Whether the user may select more than one choice.
    pub multi_select: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskUserToolOption {
    /// A concise choice label.
    pub label: String,
    /// A short explanation of the choice and its implications.
    pub description: String,
}

pub struct AskUserTool;

impl AgentTool for AskUserTool {
    type Input = AskUserToolInput;
    type Output = String;

    const NAME: &'static str = "ask_user";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Other
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        input
            .ok()
            .and_then(|input| input.questions.into_iter().next())
            .map(|question| question.question.into())
            .unwrap_or_else(|| "Ask user".into())
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|error| error.to_string())?;
            validate_input(&input)?;
            let request = acp_thread::AskUserRequest {
                questions: input
                    .questions
                    .into_iter()
                    .map(|question| acp_thread::AskUserQuestion {
                        question: question.question,
                        header: question.header,
                        options: question
                            .options
                            .into_iter()
                            .map(|option| acp_thread::AskUserOption {
                                label: option.label,
                                description: option.description,
                            })
                            .collect(),
                        multi_select: question.multi_select,
                    })
                    .collect(),
            };

            let response_task = cx.update(|cx| event_stream.ask_user(request, cx));
            let response = response_task.await.map_err(|error| error.to_string())?;
            serde_json::to_string(&response).map_err(|error| error.to_string())
        })
    }
}

fn validate_input(input: &AskUserToolInput) -> Result<(), String> {
    if !(1..=4).contains(&input.questions.len()) {
        return Err("ask_user requires between one and four questions".to_string());
    }
    for (index, question) in input.questions.iter().enumerate() {
        if question.question.trim().is_empty() {
            return Err(format!("ask_user question {} is empty", index + 1));
        }
        let header_length = question.header.chars().count();
        if header_length == 0 || header_length > 12 {
            return Err(format!(
                "ask_user question {} header must contain between 1 and 12 characters",
                index + 1
            ));
        }
        if !(2..=4).contains(&question.options.len()) {
            return Err(format!(
                "ask_user question {} requires between two and four options",
                index + 1
            ));
        }
        if question
            .options
            .iter()
            .any(|option| option.label.trim().is_empty())
        {
            return Err(format!(
                "ask_user question {} contains an empty option label",
                index + 1
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_claude_code_question_limits() {
        let input = AskUserToolInput {
            questions: vec![AskUserToolQuestion {
                question: "Which approach should be used?".to_string(),
                header: "Approach".to_string(),
                options: vec![
                    AskUserToolOption {
                        label: "A".to_string(),
                        description: "Use A".to_string(),
                    },
                    AskUserToolOption {
                        label: "B".to_string(),
                        description: "Use B".to_string(),
                    },
                ],
                multi_select: false,
            }],
        };
        assert_eq!(validate_input(&input), Ok(()));
    }
}
