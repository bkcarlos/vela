use crate::{
    NewFile, Open, OpenMode, PathList, RecentWorkspace, SerializedWorkspaceLocation, Workspace,
    WorkspaceSettings,
    item::{Item, ItemEvent},
    persistence::WorkspaceDb,
};
use git::Clone as GitClone;
use gpui::WeakEntity;
use gpui::{
    Action, App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    ParentElement, Render, Styled, Task, TaskExt, Window, actions,
};
use menu::{SelectNext, SelectPrevious};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::{DefaultOpenBehavior, Settings};
use std::path::Path;
use ui::{ButtonLike, prelude::*};
use util::ResultExt;
use vela_actions::{Extensions, OpenRecent, OpenRemote, OpenSettings, command_palette};

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, JsonSchema, Action)]
#[action(namespace = welcome)]
#[serde(transparent)]
pub struct OpenRecentProject {
    pub index: usize,
}

actions!(
    vela,
    [
        /// Show the Vela welcome screen
        ShowWelcome
    ]
);

#[derive(IntoElement)]
struct SectionHeader {
    title: SharedString,
}

impl SectionHeader {
    fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Label::new(self.title)
            .size(LabelSize::Small)
            .color(Color::Muted)
    }
}

#[derive(IntoElement)]
struct SectionButton {
    label: SharedString,
    icon: IconName,
    action: Box<dyn Action>,
    tab_index: usize,
    focus_handle: FocusHandle,
}

impl SectionButton {
    fn new(
        label: impl Into<SharedString>,
        icon: IconName,
        action: &dyn Action,
        tab_index: usize,
        focus_handle: FocusHandle,
    ) -> Self {
        Self {
            label: label.into(),
            icon,
            action: action.boxed_clone(),
            tab_index,
            focus_handle,
        }
    }
}

impl RenderOnce for SectionButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let id = format!("welcome-button-{}-{}", self.label, self.tab_index);

        ButtonLike::new(id)
            .tab_index(self.tab_index as isize)
            .size(ButtonSize::Medium)
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Icon::new(self.icon)
                            .color(Color::Accent)
                            .size(IconSize::Small),
                    )
                    .child(Label::new(self.label).color(Color::Accent)),
            )
            .on_click(move |_, window, cx| {
                self.focus_handle.dispatch_action(&*self.action, window, cx)
            })
    }
}

#[derive(IntoElement)]
struct RecentProjectButton {
    name: SharedString,
    path: SharedString,
    action: Box<dyn Action>,
    tab_index: usize,
    focus_handle: FocusHandle,
}

impl RenderOnce for RecentProjectButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let id = format!("welcome-recent-{}-{}", self.name, self.tab_index);

        ButtonLike::new(id)
            .tab_index(self.tab_index as isize)
            .size(ButtonSize::Compact)
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_3()
                    .child(Label::new(self.name).color(Color::Accent).truncate())
                    .child(
                        Label::new(self.path)
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .on_click(move |_, window, cx| {
                self.focus_handle.dispatch_action(&*self.action, window, cx)
            })
    }
}

struct SectionEntry {
    icon: IconName,
    title: &'static str,
    action: &'static dyn Action,
}

impl SectionEntry {
    fn render(&self, button_index: usize, focus: &FocusHandle) -> impl IntoElement {
        SectionButton::new(
            self.title,
            self.icon,
            self.action,
            button_index,
            focus.clone(),
        )
    }
}

const START: Section<4> = Section {
    title: "Start",
    entries: [
        SectionEntry {
            icon: IconName::FolderOpen,
            title: "Open File or Folder...",
            action: &Open::DEFAULT,
        },
        SectionEntry {
            icon: IconName::Plus,
            title: "New File...",
            action: &NewFile,
        },
        SectionEntry {
            icon: IconName::GitBranch,
            title: "Clone Git Repository...",
            action: &GitClone,
        },
        SectionEntry {
            icon: IconName::Server,
            title: "Connect to...",
            action: &OpenRemote {
                from_existing_connection: false,
                create_new_window: None,
            },
        },
    ],
};

struct Section<const COLS: usize> {
    title: &'static str,
    entries: [SectionEntry; COLS],
}

impl<const COLS: usize> Section<COLS> {
    fn render(self, index_offset: usize, focus: &FocusHandle) -> impl IntoElement {
        v_flex()
            .w_full()
            .flex_none()
            .gap_2()
            .child(SectionHeader::new(self.title))
            .children(
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| entry.render(index_offset + index, focus)),
            )
    }
}

pub struct WelcomePage {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    #[allow(dead_code)]
    fallback_to_recent_projects: bool,
    recent_workspaces: Option<Vec<RecentWorkspace>>,
}

impl WelcomePage {
    pub fn new(
        workspace: WeakEntity<Workspace>,
        fallback_to_recent_projects: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        cx.on_focus(&focus_handle, window, |_, _, cx| cx.notify())
            .detach();

        let db = WorkspaceDb::global(cx);
        cx.spawn_in(window, {
            let workspace = workspace.clone();
            async move |this: WeakEntity<Self>, cx| {
                let Some(fs) = workspace
                    .read_with(cx, |workspace, _| workspace.app_state().fs.clone())
                    .log_err()
                else {
                    return;
                };
                let workspaces = db
                    .recent_project_workspaces(fs.as_ref())
                    .await
                    .log_err()
                    .unwrap_or_default();

                this.update(cx, |this, cx| {
                    this.recent_workspaces = Some(workspaces);
                    cx.notify();
                })
                .log_err();
            }
        })
        .detach();

        let page = WelcomePage {
            workspace: workspace.clone(),
            focus_handle,
            fallback_to_recent_projects,
            recent_workspaces: None,
        };
        // Callers may still be constructing or updating the workspace.
        cx.defer_in(window, move |_, window, cx| {
            if let Some(workspace) = workspace.upgrade() {
                workspace.update(cx, |workspace, cx| {
                    close_agent_panel(workspace, window, cx);
                });
            }
        });
        page
    }

    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
        cx.notify();
    }

    fn select_previous(&mut self, _: &SelectPrevious, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
        cx.notify();
    }

    fn open_recent_project(
        &mut self,
        action: &OpenRecentProject,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(recent_workspaces) = &self.recent_workspaces {
            if let Some(workspace) = recent_workspaces.get(action.index) {
                let is_local = matches!(workspace.location, SerializedWorkspaceLocation::Local);

                if is_local {
                    let paths = workspace.paths.paths().to_vec();
                    let open_mode = match WorkspaceSettings::get_global(cx).default_open_behavior {
                        DefaultOpenBehavior::ExistingWindow => OpenMode::Activate,
                        DefaultOpenBehavior::NewWindow => OpenMode::NewWindow,
                    };
                    self.workspace
                        .update(cx, |workspace, cx| {
                            workspace
                                .open_workspace_for_paths(open_mode, paths, window, cx)
                                .detach_and_log_err(cx);
                        })
                        .log_err();
                } else {
                    window.dispatch_action(OpenRecent::default().boxed_clone(), cx);
                }
            }
        }
    }

    fn render_recent_project(
        &self,
        project_index: usize,
        tab_index: usize,
        paths: &PathList,
    ) -> impl IntoElement {
        RecentProjectButton {
            name: project_name(paths).into(),
            path: project_path_display(paths).into(),
            action: OpenRecentProject {
                index: project_index,
            }
            .boxed_clone(),
            tab_index,
            focus_handle: self.focus_handle.clone(),
        }
    }

    fn render_recent_section(&self, start_tab_index: usize) -> impl IntoElement {
        let recents = self
            .recent_workspaces
            .as_ref()
            .into_iter()
            .flatten()
            .take(5)
            .enumerate()
            .map(|(index, workspace)| {
                self.render_recent_project(
                    index,
                    start_tab_index + index,
                    &workspace.identity_paths,
                )
            })
            .collect::<Vec<_>>();

        let more_tab_index = start_tab_index + recents.len();
        let focus = self.focus_handle.clone();
        let is_empty = recents.is_empty();

        v_flex()
            .w_full()
            .gap_1()
            .child(SectionHeader::new("Recent"))
            .when(is_empty, |this| {
                this.child(
                    Label::new("You have no recent folders")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .children(recents)
            .when(!is_empty, |this| {
                this.child(
                    ButtonLike::new("welcome-recent-more")
                        .tab_index(more_tab_index as isize)
                        .size(ButtonSize::Compact)
                        .child(Label::new("More...").color(Color::Accent))
                        .on_click(move |_, window, cx| {
                            focus.dispatch_action(&OpenRecent::default(), window, cx)
                        }),
                )
            })
    }

    fn render_tools(&self, start_tab_index: usize) -> impl IntoElement {
        Section {
            title: "Tools",
            entries: [
                SectionEntry {
                    icon: IconName::Keyboard,
                    title: "Command Palette",
                    action: &command_palette::Toggle,
                },
                SectionEntry {
                    icon: IconName::Settings,
                    title: "Settings",
                    action: &OpenSettings,
                },
                SectionEntry {
                    icon: IconName::Blocks,
                    title: "Extensions",
                    action: &Extensions {
                        category_filter: None,
                        id: None,
                    },
                },
            ],
        }
        .render(start_tab_index, &self.focus_handle)
    }
}

impl Render for WelcomePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let start_section = START;
        let recent_tab_index = start_section.entries.len();
        let tools_tab_index = recent_tab_index + 10;

        v_flex()
            .id("welcome-page")
            .key_context("Welcome")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::open_recent_project))
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().colors().editor_background)
            .items_center()
            .p_6()
            .child(
                v_flex()
                    .id("welcome-content")
                    .w_full()
                    .max_w(px(560.))
                    .flex_none()
                    .py_8()
                    .gap_6()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_3xl()
                                    .text_color(cx.theme().colors().text)
                                    .child("Vela"),
                            )
                            .child(
                                Label::new("Open a project. Start building.")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(start_section.render(0, &self.focus_handle))
                    .child(self.render_recent_section(recent_tab_index))
                    .child(self.render_tools(tools_tab_index)),
            )
    }
}

impl EventEmitter<ItemEvent> for WelcomePage {}

impl Focusable for WelcomePage {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for WelcomePage {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Welcome".into()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("New Welcome Page Opened")
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        close_agent_panel(workspace, window, cx);
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(crate::item::ItemEvent)) {
        f(*event)
    }
}

fn close_agent_panel(workspace: &mut Workspace, window: &mut Window, cx: &mut gpui::App) {
    if let Some(position) = workspace.agent_panel_position(cx) {
        workspace
            .dock_at_position(position)
            .update(cx, |dock, cx| dock.set_open(false, window, cx));
    }
}

fn project_path_display(paths: &PathList) -> String {
    let Some(path) = paths.paths().first() else {
        return String::new();
    };
    let parent = path.parent().unwrap_or(path.as_ref());
    if let Some(home) = std::env::var_os("HOME") {
        let home = Path::new(&home);
        if let Ok(stripped) = parent.strip_prefix(home) {
            let rest = stripped.display().to_string();
            return if rest.is_empty() {
                "~".to_string()
            } else {
                format!("~/{rest}")
            };
        }
    }
    parent.display().to_string()
}

impl crate::SerializableItem for WelcomePage {
    fn serialized_item_kind() -> &'static str {
        "WelcomePage"
    }

    fn cleanup(
        workspace_id: crate::WorkspaceId,
        alive_items: Vec<crate::ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<gpui::Result<()>> {
        crate::delete_unloaded_items(
            alive_items,
            workspace_id,
            "welcome_pages",
            &persistence::WelcomePagesDb::global(cx),
            cx,
        )
    }

    fn deserialize(
        _project: Entity<project::Project>,
        workspace: gpui::WeakEntity<Workspace>,
        workspace_id: crate::WorkspaceId,
        item_id: crate::ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<gpui::Result<Entity<Self>>> {
        if persistence::WelcomePagesDb::global(cx)
            .get_welcome_page(item_id, workspace_id)
            .ok()
            .is_some_and(|is_open| is_open)
        {
            Task::ready(Ok(
                cx.new(|cx| WelcomePage::new(workspace, false, window, cx))
            ))
        } else {
            Task::ready(Err(anyhow::anyhow!("No welcome page to deserialize")))
        }
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: crate::ItemId,
        _closing: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Task<gpui::Result<()>>> {
        let workspace_id = workspace.database_id()?;
        let db = persistence::WelcomePagesDb::global(cx);
        Some(cx.background_spawn(
            async move { db.save_welcome_page(item_id, workspace_id, true).await },
        ))
    }

    fn should_serialize(&self, event: &Self::Event) -> bool {
        event == &ItemEvent::UpdateTab
    }
}

mod persistence {
    use crate::WorkspaceDb;
    use db::{
        query,
        sqlez::{domain::Domain, thread_safe_connection::ThreadSafeConnection},
        sqlez_macros::sql,
    };

    pub struct WelcomePagesDb(ThreadSafeConnection);

    impl Domain for WelcomePagesDb {
        const NAME: &str = stringify!(WelcomePagesDb);

        const MIGRATIONS: &[&str] = (&[sql!(
                    CREATE TABLE welcome_pages (
                        workspace_id INTEGER,
                        item_id INTEGER UNIQUE,
                        is_open INTEGER DEFAULT FALSE,

                        PRIMARY KEY(workspace_id, item_id),
                        FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                        ON DELETE CASCADE
                    ) STRICT;
        )]);
    }

    db::static_connection!(WelcomePagesDb, [WorkspaceDb]);

    impl WelcomePagesDb {
        query! {
            pub async fn save_welcome_page(
                item_id: crate::ItemId,
                workspace_id: crate::WorkspaceId,
                is_open: bool
            ) -> Result<()> {
                INSERT OR REPLACE INTO welcome_pages(item_id, workspace_id, is_open)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub fn get_welcome_page(
                item_id: crate::ItemId,
                workspace_id: crate::WorkspaceId
            ) -> Result<bool> {
                SELECT is_open
                FROM welcome_pages
                WHERE item_id = ? AND workspace_id = ?
            }
        }
    }
}

fn project_name(paths: &PathList) -> String {
    let joined = paths
        .paths()
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    if joined.is_empty() {
        "Untitled".to_string()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_empty() {
        let paths = PathList::new::<&str>(&[]);
        assert_eq!(project_name(&paths), "Untitled");
    }

    #[test]
    fn test_project_name_single() {
        let paths = PathList::new(&["/home/user/my-project"]);
        assert_eq!(project_name(&paths), "my-project");
    }

    #[test]
    fn test_project_name_multiple() {
        // PathList sorts lexicographically, so filenames appear in alpha order
        let paths = PathList::new(&["/home/user/vela", "/home/user/api"]);
        assert_eq!(project_name(&paths), "api, vela");
    }

    #[test]
    fn test_project_name_root_path_filtered() {
        // A bare root "/" has no file_name(), falls back to "Untitled"
        let paths = PathList::new(&["/"]);
        assert_eq!(project_name(&paths), "Untitled");
    }
}
