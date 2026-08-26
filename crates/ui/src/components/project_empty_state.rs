use crate::{Divider, DividerColor, KeyBinding, prelude::*};
use gpui::{ClickEvent, FocusHandle};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct ProjectEmptyState {
    label: SharedString,
    focus_handle: FocusHandle,
    open_project_key_binding: KeyBinding,
    on_new_file: Option<ClickHandler>,
    on_open_project: Option<ClickHandler>,
    on_clone_repo: Option<ClickHandler>,
}

impl ProjectEmptyState {
    pub fn new(
        label: impl Into<SharedString>,
        focus_handle: FocusHandle,
        open_project_key_binding: KeyBinding,
    ) -> Self {
        Self {
            label: label.into(),
            focus_handle,
            open_project_key_binding,
            on_new_file: None,
            on_open_project: None,
            on_clone_repo: None,
        }
    }

    pub fn on_new_file(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_new_file = Some(Box::new(handler));
        self
    }

    pub fn on_open_project(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_project = Some(Box::new(handler));
        self
    }

    pub fn on_clone_repo(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clone_repo = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ProjectEmptyState {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let id = format!("empty-state-{}", self.label);
        let label = format!("Choose a start action to use the {}", self.label);

        v_flex()
            .id(id)
            .p_4()
            .size_full()
            .items_center()
            .justify_center()
            .track_focus(&self.focus_handle)
            .child(
                v_flex()
                    .w_48()
                    .max_w_full()
                    .gap_1()
                    .child(
                        div()
                            .text_center()
                            .mb_2()
                            .child(Label::new(label).size(LabelSize::Small).color(Color::Muted)),
                    )
                    .when_some(self.on_new_file, |this, handler| {
                        this.child(
                            Button::new("new_file", "New File")
                                .full_width()
                                .style(ButtonStyle::Outlined)
                                .start_icon(Icon::new(IconName::Plus).size(IconSize::Small))
                                .on_click(handler),
                        )
                    })
                    .child(
                        Button::new("open_project", "Open")
                            .full_width()
                            .style(ButtonStyle::Filled)
                            .start_icon(Icon::new(IconName::FolderOpen).size(IconSize::Small))
                            .key_binding(self.open_project_key_binding)
                            .when_some(self.on_open_project, |button, handler| {
                                button.on_click(handler)
                            }),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Divider::horizontal().color(DividerColor::Border))
                            .child(Label::new("or").size(LabelSize::XSmall).color(Color::Muted))
                            .child(Divider::horizontal().color(DividerColor::Border)),
                    )
                    .child(
                        Button::new("clone_repo", "Clone Git Repository")
                            .full_width()
                            .style(ButtonStyle::Outlined)
                            .start_icon(Icon::new(IconName::GitBranch).size(IconSize::Small))
                            .when_some(self.on_clone_repo, |button, handler| {
                                button.on_click(handler)
                            }),
                    ),
            )
    }
}
