mod agent_api_keys_onboarding;
mod agent_panel_onboarding_card;
mod agent_panel_onboarding_content;
mod edit_prediction_onboarding_content;

pub use agent_api_keys_onboarding::{ApiKeysWithProviders, ApiKeysWithoutProviders};
pub use agent_panel_onboarding_card::AgentPanelOnboardingCard;
pub use agent_panel_onboarding_content::AgentPanelOnboarding;
pub use edit_prediction_onboarding_content::EditPredictionOnboarding;

use std::sync::Arc;

use client::{Client, UserStore};
use gpui::{AnyElement, Entity, IntoElement, ParentElement};
use ui::{RegisterComponent, Tooltip, prelude::*};

#[derive(RegisterComponent, IntoElement)]
pub struct VelaAiOnboarding {
    settings_path: &'static str,
    pub dismiss_onboarding: Option<Arc<dyn Fn(&mut Window, &mut App)>>,
}

impl VelaAiOnboarding {
    pub fn new(
        _client: Arc<Client>,
        _user_store: &Entity<UserStore>,
        _continue_with_vela_ai: Arc<dyn Fn(&mut Window, &mut App)>,
        _cx: &mut App,
    ) -> Self {
        Self {
            settings_path: "llm_providers",
            dismiss_onboarding: None,
        }
    }

    pub fn with_settings_path(mut self, settings_path: &'static str) -> Self {
        self.settings_path = settings_path;
        self
    }

    pub fn with_dismiss(
        mut self,
        dismiss_callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.dismiss_onboarding = Some(Arc::new(dismiss_callback));
        self
    }

    fn render_dismiss_button(&self) -> Option<AnyElement> {
        self.dismiss_onboarding.as_ref().map(|dismiss_callback| {
            let callback = dismiss_callback.clone();

            h_flex()
                .absolute()
                .top_0()
                .right_0()
                .child(
                    IconButton::new("dismiss_onboarding", IconName::Close)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Dismiss"))
                        .on_click(move |_, window, cx| {
                            telemetry::event!("Banner Dismissed", source = "AI Onboarding",);
                            callback(window, cx)
                        }),
                )
                .into_any_element()
        })
    }
}

impl RenderOnce for VelaAiOnboarding {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let settings_path = self.settings_path;
        v_flex()
            .relative()
            .gap_3()
            .child(Headline::new("Configure your AI provider"))
            .child(
                Label::new(
                    "Use your own API key or a local model. Configure a provider to get started.",
                )
                .color(Color::Muted),
            )
            .child(
                Button::new("configure-model-provider", "Configure Models")
                    .full_width()
                    .style(ButtonStyle::Tinted(ui::TintColor::Accent))
                    .on_click(move |_, window, cx| {
                        window.dispatch_action(
                            Box::new(vela_actions::OpenSettingsAt {
                                path: settings_path.to_string(),
                                target: None,
                            }),
                            cx,
                        );
                    }),
            )
            .children(self.render_dismiss_button())
    }
}

impl Component for VelaAiOnboarding {
    fn scope() -> ComponentScope {
        ComponentScope::Onboarding
    }

    fn name() -> &'static str {
        "Agent New User Onboarding"
    }

    fn description() -> &'static str {
        "The onboarding surface shown to new agent panel users, \
        guiding them through configuring their own model provider."
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> AnyElement {
        AgentPanelOnboardingCard::new()
            .child(Self {
                settings_path: "llm_providers",
                dismiss_onboarding: None,
            })
            .into_any_element()
    }
}
