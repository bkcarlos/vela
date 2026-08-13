use agent_client_protocol::schema::v1 as acp;
use gpui::{App, SharedString, Task};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{AgentTool, ToolCallEventStream, ToolInput};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskUserToolInput {
    /// The question that needs the user's decision.
    pub question: String,
    /// The possible answers. The selected answer is returned to the model.
    pub options: Vec<String>,
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
        match input {
            Ok(input) => input.question.into(),
            Err(_) => "Ask user".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|error| error.to_string())?;
            if input.options.is_empty() {
                return Err("ask_user requires at least one option".to_string());
            }

            let options = input
                .options
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    acp::PermissionOption::new(
                        acp::PermissionOptionId::new(format!("option-{index}")),
                        label,
                        acp::PermissionOptionKind::AllowOnce,
                    )
                })
                .collect();

            let decision_task = cx.update(|cx| {
                event_stream.prompt_for_decision(
                    Some("The agent needs your input".to_string()),
                    Some(input.question),
                    options,
                    cx,
                )
            });
            decision_task
                .await
                .map_err(|error| error.to_string())
                .and_then(|option_id| {
                    let index = option_id
                        .0
                        .strip_prefix("option-")
                        .and_then(|index| index.parse::<usize>().ok());
                    index
                        .and_then(|index| input.options.get(index).cloned())
                        .ok_or_else(|| "The selected answer was not recognized".to_string())
                })
        })
    }
}
