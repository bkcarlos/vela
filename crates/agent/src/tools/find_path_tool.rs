use crate::{
    AgentTool, ExplicitToolPermissionDecision, ToolCallEventStream, ToolInput,
    decide_permission_for_paths_from_explicit_rules,
};
use acp_thread::MentionUri;
use agent_client_protocol::schema::v1 as acp;
use anyhow::{Context as _, Result, anyhow};
use futures::{FutureExt as _, StreamExt as _};
use gpui::{App, AppContext, Entity, SharedString, Task};
use language_model::LanguageModelToolResultContent;
use project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::Settings;
use std::fmt::Write;
use std::{
    cmp,
    path::{Path, PathBuf},
    sync::Arc,
};
use util::{
    paths::PathMatcher,
    rel_path::{RelPath, RelPathBuf},
};

/// Find file paths that match a given pattern.
///
/// - Supports glob patterns like "**/*.js" or "src/**/*.ts"
/// - Absolute globs outside project worktrees require confirmation unless explicitly allowed, and are scoped to their fixed directory prefix
/// - Returns matching file paths sorted alphabetically
/// - Prefer the `grep` tool to this tool when searching for symbols unless you have specific information about paths.
/// - Use this tool when you need to find files by name patterns
/// - Results are paginated with 50 matches per page. Use the optional 'offset' parameter to request subsequent pages.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FindPathToolInput {
    /// The glob to match against paths. Project paths use the indexed worktrees; absolute globs outside the project require confirmation unless explicitly allowed.
    ///
    /// <example>
    /// If the project has the following root directories:
    ///
    /// - directory1/a/something.txt
    /// - directory2/a/things.txt
    /// - directory3/a/other.txt
    ///
    /// You can get back the first two paths by providing a glob of "*thing*.txt"
    /// </example>
    pub glob: String,
    /// Optional starting position for paginated results (0-based).
    /// When not provided, starts from the beginning.
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FindPathToolOutput {
    Success {
        offset: usize,
        current_matches_page: Vec<PathBuf>,
        all_matches_len: usize,
    },
    Error {
        error: String,
    },
}

impl From<FindPathToolOutput> for LanguageModelToolResultContent {
    fn from(output: FindPathToolOutput) -> Self {
        match output {
            FindPathToolOutput::Success {
                offset,
                current_matches_page,
                all_matches_len,
            } => {
                if current_matches_page.is_empty() {
                    "No matches found".into()
                } else {
                    let mut llm_output = format!("Found {} total matches.", all_matches_len);
                    if all_matches_len > RESULTS_PER_PAGE {
                        write!(
                            &mut llm_output,
                            "\nShowing results {}-{} (provide 'offset' parameter for more results):",
                            offset + 1,
                            offset + current_matches_page.len()
                        )
                        .ok();
                    }

                    for mat in current_matches_page {
                        write!(&mut llm_output, "\n{}", mat.display()).ok();
                    }

                    llm_output.into()
                }
            }
            FindPathToolOutput::Error { error } => error.into(),
        }
    }
}

const RESULTS_PER_PAGE: usize = 50;

pub struct FindPathTool {
    project: Entity<Project>,
}

impl FindPathTool {
    pub fn new(project: Entity<Project>) -> Self {
        Self { project }
    }
}

impl AgentTool for FindPathTool {
    type Input = FindPathToolInput;
    type Output = FindPathToolOutput;

    const NAME: &'static str = "find_path";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Search
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        let mut title = "Find paths".to_string();
        if let Ok(input) = input {
            title.push_str(&format!(" matching “`{}`”", input.glob));
        }
        title.into()
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let project = self.project.clone();
        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|e| FindPathToolOutput::Error {
                error: e.to_string(),
            })?;

            let external_search_root =
                cx.update(|cx| external_search_root(&input.glob, &project, cx));
            let search_paths_task = if let Some(external_search_root) = external_search_root {
                let windows_path_style =
                    project.read_with(cx, |project, cx| project.path_style(cx).is_windows());
                let initial_permission_inputs = vec![
                    input.glob.clone(),
                    external_search_root.to_string_lossy().into_owned(),
                ];
                let initial_permission_decision = cx.update(|cx| {
                    decide_permission_for_paths_from_explicit_rules(
                        Self::NAME,
                        &initial_permission_inputs,
                        windows_path_style,
                        agent_settings::AgentSettings::get_global(cx),
                    )
                });
                let initial_confirmation_required = match initial_permission_decision {
                    ExplicitToolPermissionDecision::Deny(reason) => {
                        return Err(FindPathToolOutput::Error { error: reason });
                    }
                    ExplicitToolPermissionDecision::Confirm => true,
                    ExplicitToolPermissionDecision::Allow
                    | ExplicitToolPermissionDecision::NoMatch => false,
                };

                let fs = project.read_with(cx, |project, _cx| project.fs().clone());
                let metadata = cx.background_spawn({
                    let external_search_root = external_search_root.clone();
                    let fs = fs.clone();
                    async move { fs.metadata(&external_search_root).await }
                });
                let root_exists = futures::select! {
                    result = metadata.fuse() => result.map_err(|error| FindPathToolOutput::Error {
                        error: format!("Reading external search root {}: {error}", external_search_root.display()),
                    })?.is_some(),
                    _ = event_stream.cancelled_by_user().fuse() => {
                        return Err(FindPathToolOutput::Error { error: "Path search cancelled by user".to_string() });
                    }
                };
                if !root_exists {
                    return Ok(FindPathToolOutput::Success {
                        offset: input.offset,
                        current_matches_page: Vec::new(),
                        all_matches_len: 0,
                    });
                }

                let canonicalize = cx.background_spawn({
                    let external_search_root = external_search_root.clone();
                    async move { fs.canonicalize(&external_search_root).await }
                });
                let canonical_search_root = futures::select! {
                    result = canonicalize.fuse() => result.map_err(|error| FindPathToolOutput::Error {
                        error: format!("Resolving external search root {}: {error}", external_search_root.display()),
                    })?,
                    _ = event_stream.cancelled_by_user().fuse() => {
                        return Err(FindPathToolOutput::Error { error: "Path search cancelled by user".to_string() });
                    }
                };
                let canonical_glob = cx
                    .update(|cx| {
                        canonical_external_glob(&input.glob, &canonical_search_root, &project, cx)
                    })
                    .map_err(|error| FindPathToolOutput::Error {
                        error: error.to_string(),
                    })?;
                let canonical_root_permission = canonical_search_root.to_string_lossy();
                let canonical_root_permission = if windows_path_style {
                    canonical_root_permission.replace('\\', "/")
                } else {
                    canonical_root_permission.into_owned()
                };
                let canonical_glob_permission = if windows_path_style {
                    canonical_glob.replace('\\', "/")
                } else {
                    canonical_glob.clone()
                };
                let canonical_permission_inputs = vec![
                    canonical_root_permission.clone(),
                    canonical_glob_permission,
                ];
                let permission_decision = cx.update(|cx| {
                    decide_permission_for_paths_from_explicit_rules(
                        Self::NAME,
                        &canonical_permission_inputs,
                        windows_path_style,
                        agent_settings::AgentSettings::get_global(cx),
                    )
                });
                match permission_decision {
                    ExplicitToolPermissionDecision::Deny(reason) => {
                        return Err(FindPathToolOutput::Error { error: reason });
                    }
                    ExplicitToolPermissionDecision::Allow
                        if !initial_confirmation_required => {}
                    ExplicitToolPermissionDecision::Allow
                    | ExplicitToolPermissionDecision::Confirm
                    | ExplicitToolPermissionDecision::NoMatch => {
                        let authorize = cx.update(|cx| {
                            let display_root = canonical_search_root.display();
                            let context = crate::ToolPermissionContext::new(
                                Self::NAME,
                                vec![canonical_root_permission.clone()],
                            );
                            event_stream.authorize_always_prompt(
                                format!("Search outside the project in `{display_root}`"),
                                context,
                                cx,
                            )
                        });
                        futures::select! {
                            result = authorize.fuse() => result.map_err(|error| FindPathToolOutput::Error {
                                error: error.to_string(),
                            })?,
                            _ = event_stream.cancelled_by_user().fuse() => {
                                return Err(FindPathToolOutput::Error { error: "Path search cancelled by user".to_string() });
                            }
                        }
                    }
                }
                cx.update(|cx| {
                    search_external_paths(
                        &canonical_glob,
                        canonical_search_root,
                        project,
                        cx,
                    )
                })
            } else {
                cx.update(|cx| search_paths(&input.glob, project, cx))
            };

            let matches = futures::select! {
                result = search_paths_task.fuse() => result.map_err(|error| FindPathToolOutput::Error { error: error.to_string() })?,
                _ = event_stream.cancelled_by_user().fuse() => {
                    return Err(FindPathToolOutput::Error { error: "Path search cancelled by user".to_string() });
                }
            };
            let paginated_matches: &[PathBuf] = &matches[cmp::min(input.offset, matches.len())
                ..cmp::min(input.offset + RESULTS_PER_PAGE, matches.len())];

            event_stream.update_fields(
                acp::ToolCallUpdateFields::new()
                    .title(if paginated_matches.is_empty() {
                        "No matches".into()
                    } else if paginated_matches.len() == 1 {
                        "1 match".into()
                    } else {
                        format!("{} matches", paginated_matches.len())
                    })
                    .content(
                        paginated_matches
                            .iter()
                            .map(|path| {
                                let uri = MentionUri::File {
                                    abs_path: path.clone(),
                                };
                                acp::ToolCallContent::Content(acp::Content::new(
                                    acp::ContentBlock::ResourceLink(acp::ResourceLink::new(
                                        path.to_string_lossy(),
                                        uri.to_uri().to_string(),
                                    )),
                                ))
                            })
                            .collect::<Vec<_>>(),
                    ),
            );

            Ok(FindPathToolOutput::Success {
                offset: input.offset,
                current_matches_page: paginated_matches.to_vec(),
                all_matches_len: matches.len(),
            })
        })
    }
}

fn search_paths(glob: &str, project: Entity<Project>, cx: &mut App) -> Task<Result<Vec<PathBuf>>> {
    let path_style = project.read(cx).path_style(cx);
    let snapshots: Vec<_> = project
        .read(cx)
        .worktrees(cx)
        .map(|worktree| worktree.read(cx).snapshot())
        .collect();
    // Sometimes models try to search for "". In this case, return all paths in the project.
    let glob = if glob.is_empty() { "*" } else { glob };
    let glob = if path_style.is_absolute(glob) {
        let mut closest_worktree = None;
        for snapshot in &snapshots {
            let Some(relative_glob) = path_style
                .strip_prefix(Path::new(glob), snapshot.abs_path().as_ref())
                .map(std::borrow::Cow::into_owned)
            else {
                continue;
            };

            if closest_worktree.as_ref().is_none_or(
                |(_, current_relative_glob): &(RelPathBuf, RelPathBuf)| {
                    relative_glob.len() < current_relative_glob.len()
                },
            ) {
                closest_worktree = Some((snapshot.root_name().to_owned(), relative_glob));
            }
        }

        if let Some((root_name, relative_glob)) = closest_worktree {
            root_name
                .join(&relative_glob)
                .display(path_style)
                .into_owned()
        } else {
            glob.to_owned()
        }
    } else {
        glob.to_owned()
    };

    let path_matcher = match PathMatcher::new([&glob], path_style) {
        Ok(matcher) => matcher,
        Err(err) => return Task::ready(Err(anyhow!("Invalid glob: {err}"))),
    };
    let literal_prefix = literal_glob_prefix(&glob, path_style.separators_ch());
    let root_names: Vec<RelPathBuf> = snapshots
        .iter()
        .map(|snapshot| snapshot.root_name().to_owned())
        .collect();

    cx.background_spawn(async move {
        let mut results = Vec::new();
        for snapshot in snapshots {
            let search_root = if let Some(literal_prefix) = &literal_prefix {
                if let Ok(relative_prefix) = literal_prefix.strip_prefix(snapshot.root_name()) {
                    relative_prefix.to_owned()
                } else if root_names
                    .iter()
                    .any(|root_name| !root_name.is_empty() && literal_prefix.starts_with(root_name))
                {
                    continue;
                } else {
                    RelPathBuf::new()
                }
            } else {
                RelPathBuf::new()
            };

            for entry in snapshot.traverse_from_path(true, true, false, &search_root) {
                if !entry.path.starts_with(&search_root) {
                    break;
                }
                if path_matcher.is_match(&snapshot.root_name().join(&entry.path)) {
                    results.push(snapshot.absolutize(&entry.path));
                }
            }
        }

        Ok(results)
    })
}

fn external_search_root(glob: &str, project: &Entity<Project>, cx: &App) -> Option<PathBuf> {
    let path_style = project.read(cx).path_style(cx);
    if glob.is_empty() || !path_style.is_absolute(glob) {
        return None;
    }

    let glob_path = Path::new(glob);
    if project.read(cx).worktrees(cx).any(|worktree| {
        path_style
            .strip_prefix(glob_path, worktree.read(cx).abs_path().as_ref())
            .is_some()
    }) {
        return None;
    }

    let (fixed_prefix, _) =
        split_glob_at_first_metachar(glob, path_style.separators_ch(), path_style.is_windows());
    Some(PathBuf::from(fixed_prefix))
}

fn canonical_external_glob(
    glob: &str,
    canonical_search_root: &Path,
    project: &Entity<Project>,
    cx: &App,
) -> Result<String> {
    let path_style = project.read(cx).path_style(cx);
    let (_, glob_suffix) =
        split_glob_at_first_metachar(glob, path_style.separators_ch(), path_style.is_windows());
    if glob_suffix.is_empty() {
        return Ok(canonical_search_root.to_string_lossy().into_owned());
    }

    path_style
        .join(canonical_search_root, glob_suffix)
        .ok_or_else(|| anyhow!("Could not scope external glob {glob:?} to its search root"))
}

fn search_external_paths(
    glob: &str,
    search_root: PathBuf,
    project: Entity<Project>,
    cx: &mut App,
) -> Task<Result<Vec<PathBuf>>> {
    let path_style = project.read(cx).path_style(cx);
    let path_matcher = match PathMatcher::new([glob], path_style) {
        Ok(path_matcher) => path_matcher,
        Err(error) => return Task::ready(Err(anyhow!("Invalid glob: {error}"))),
    };
    let fs = project.read(cx).fs().clone();
    let recursive = contains_glob_metachar(glob);

    cx.background_spawn(async move {
        let mut pending_paths = vec![search_root];
        let mut results = Vec::new();
        while let Some(path) = pending_paths.pop() {
            let canonical_path = match fs.canonicalize(&path).await {
                Ok(canonical_path) => canonical_path,
                Err(error) => {
                    log::debug!("Skipping path {} during search: {error}", path.display());
                    continue;
                }
            };
            if canonical_path != path {
                continue;
            }
            let Some(metadata) = fs
                .metadata(&canonical_path)
                .await
                .with_context(|| format!("Reading metadata for {}", canonical_path.display()))?
            else {
                continue;
            };

            if path_matcher.is_match_std_path(&canonical_path) {
                results.push(path.clone());
            }

            if recursive && metadata.is_dir && !metadata.is_symlink {
                let mut child_paths = fs
                    .read_dir(&canonical_path)
                    .await
                    .with_context(|| format!("Reading directory {}", canonical_path.display()))?;
                while let Some(child_path) = child_paths.next().await {
                    pending_paths.push(child_path.with_context(|| {
                        format!("Reading an entry in directory {}", path.display())
                    })?);
                }
            }
        }
        results.sort_unstable();
        Ok(results)
    })
}

fn literal_glob_prefix(glob: &str, separators: &[char]) -> Option<RelPathBuf> {
    let (fixed_prefix, _) = split_glob_at_first_metachar(glob, separators, false);
    let fixed_prefix = fixed_prefix.trim_matches(separators);
    if fixed_prefix.is_empty() {
        return None;
    }

    let fixed_prefix = fixed_prefix.replace('\\', "/");
    RelPath::from_unix_str(&fixed_prefix)
        .ok()
        .map(ToOwned::to_owned)
}

fn split_glob_at_first_metachar<'a>(
    glob: &'a str,
    separators: &[char],
    windows_extended_path: bool,
) -> (&'a str, &'a str) {
    let is_windows_extended_path =
        windows_extended_path && (glob.starts_with(r"\\?\") || glob.starts_with("//?/"));
    let mut component_start = 0;
    for (index, character) in glob.char_indices() {
        if !separators.contains(&character) {
            continue;
        }

        let component = &glob[component_start..index];
        let is_extended_path_marker = is_windows_extended_path && component == "?";
        if contains_glob_metachar(component) && !is_extended_path_marker {
            return (&glob[..component_start], &glob[component_start..]);
        }
        component_start = index + character.len_utf8();
    }

    let component = &glob[component_start..];
    let is_extended_path_marker = is_windows_extended_path && component == "?";
    if contains_glob_metachar(component) && !is_extended_path_marker {
        (&glob[..component_start], &glob[component_start..])
    } else {
        (glob, "")
    }
}

fn contains_glob_metachar(value: &str) -> bool {
    value.contains(['*', '?', '[', '{'])
}

#[cfg(test)]
mod test {
    use super::*;
    use fs::Fs as _;
    use gpui::TestAppContext;
    use project::{FakeFs, Project};
    use settings::SettingsStore;
    use util::path;

    #[gpui::test]
    async fn test_find_path_tool(cx: &mut TestAppContext) {
        init_test(cx);

        let fs = FakeFs::new(cx.executor());
        fs.insert_tree(
            "/root",
            serde_json::json!({
                "apple": {
                    "banana": {
                        "carrot": "1",
                    },
                    "bandana": {
                        "carbonara": "2",
                    },
                    "endive": "3"
                }
            }),
        )
        .await;
        let project = Project::test(fs.clone(), [path!("/root").as_ref()], cx).await;

        let matches = cx
            .update(|cx| search_paths("root/**/car*", project.clone(), cx))
            .await
            .unwrap();
        assert_eq!(
            matches,
            &[
                PathBuf::from(path!("/root/apple/banana/carrot")),
                PathBuf::from(path!("/root/apple/bandana/carbonara"))
            ]
        );

        let matches = cx
            .update(|cx| search_paths("**/car*", project.clone(), cx))
            .await
            .unwrap();
        assert_eq!(
            matches,
            &[
                PathBuf::from(path!("/root/apple/banana/carrot")),
                PathBuf::from(path!("/root/apple/bandana/carbonara"))
            ]
        );

        let absolute_glob = PathBuf::from(path!("/root/apple"))
            .join("**")
            .join("car*")
            .to_string_lossy()
            .into_owned();
        let matches = cx
            .update(|cx| search_paths(&absolute_glob, project.clone(), cx))
            .await
            .unwrap();
        assert_eq!(
            matches,
            &[
                PathBuf::from(path!("/root/apple/banana/carrot")),
                PathBuf::from(path!("/root/apple/bandana/carbonara"))
            ]
        );

        let external_root = cx.update(|cx| external_search_root(path!("/"), &project, cx));
        assert_eq!(external_root, Some(PathBuf::from(path!("/"))));
    }

    #[gpui::test]
    async fn test_find_path_tool_searches_authorized_external_directory(cx: &mut TestAppContext) {
        init_test(cx);

        let fs = FakeFs::new(cx.executor());
        fs.insert_tree(
            path!("/root"),
            serde_json::json!({
                "project": {},
                "outside": {
                    "nested": {
                        "result.txt": "match"
                    },
                    "other.rs": "no match"
                }
            }),
        )
        .await;
        fs.create_symlink(
            path!("/root/outside-link").as_ref(),
            PathBuf::from("outside"),
        )
        .await
        .unwrap();
        fs.create_symlink(
            path!("/root/outside/dangling-link").as_ref(),
            PathBuf::from("missing"),
        )
        .await
        .unwrap();
        let project = Project::test(fs, [path!("/root/project").as_ref()], cx).await;
        let tool = Arc::new(FindPathTool::new(project.clone()));
        let glob = PathBuf::from(path!("/root/outside-link"))
            .join("**")
            .join("*.txt")
            .to_string_lossy()
            .into_owned();
        let (event_stream, mut event_rx) = ToolCallEventStream::test();
        let task = cx.update(|cx| {
            tool.run(
                ToolInput::resolved(FindPathToolInput {
                    glob: glob.clone(),
                    offset: 0,
                }),
                event_stream,
                cx,
            )
        });

        let authorization = event_rx.expect_authorization().await;
        assert!(
            authorization
                .tool_call
                .fields
                .title
                .as_deref()
                .is_some_and(|title| title.contains(path!("/root/outside"))),
            "authorization should show the external search root"
        );
        let expected_pattern = crate::pattern_extraction::extract_directory_pattern(
            &path!("/root/outside").replace('\\', "/"),
        )
        .unwrap();
        assert!(
            matches!(
                &authorization.options,
                acp_thread::PermissionOptions::Dropdown(choices)
                    if choices.iter().any(|choice| choice.sub_patterns == [expected_pattern.clone()])
            ),
            "authorization should offer an always-allow option for the external directory"
        );
        authorization
            .response
            .send(acp_thread::SelectedPermissionOutcome::new(
                acp::PermissionOptionId::new("allow"),
                acp::PermissionOptionKind::AllowOnce,
            ))
            .unwrap();

        let result = task.await.unwrap();
        let FindPathToolOutput::Success {
            current_matches_page,
            all_matches_len,
            ..
        } = result
        else {
            panic!("expected successful external search");
        };
        assert_eq!(all_matches_len, 1);
        assert_eq!(
            current_matches_page,
            vec![PathBuf::from(path!("/root/outside/nested/result.txt"))]
        );

        cx.update(|cx| {
            let mut settings = agent_settings::AgentSettings::get_global(cx).clone();
            settings.tool_permissions.tools.insert(
                FindPathTool::NAME.into(),
                agent_settings::ToolRules {
                    always_allow: vec![
                        agent_settings::CompiledRegex::new(&expected_pattern, false).unwrap(),
                    ],
                    ..Default::default()
                },
            );
            agent_settings::AgentSettings::override_global(settings, cx);
        });
        let tool = Arc::new(FindPathTool::new(project));
        let (event_stream, mut event_rx) = ToolCallEventStream::test();
        let result = cx
            .update(|cx| {
                tool.run(
                    ToolInput::resolved(FindPathToolInput { glob, offset: 0 }),
                    event_stream,
                    cx,
                )
            })
            .await;
        assert!(result.is_ok(), "explicit allow rule should permit search");
        assert!(
            !matches!(
                event_rx.try_recv(),
                Ok(Ok(crate::ThreadEvent::ToolCallAuthorization(_)))
            ),
            "explicit allow rule should skip repeated authorization"
        );
    }

    #[test]
    fn test_literal_glob_prefix() {
        assert_eq!(
            literal_glob_prefix("root/apple/**/car*", &['/']),
            Some(RelPath::new_test("root/apple").into_owned())
        );
        assert_eq!(
            literal_glob_prefix("root/apple/[bc]*", &['/']),
            Some(RelPath::new_test("root/apple").into_owned())
        );
        assert_eq!(literal_glob_prefix("**/car*", &['/']), None);

        assert_eq!(
            split_glob_at_first_metachar("/root/**/*.rs", &['/'], false),
            ("/root/", "**/*.rs")
        );
        assert_eq!(
            split_glob_at_first_metachar(r"C:\Users\*\*.rs", &['\\', '/'], true),
            (r"C:\Users\", r"*\*.rs")
        );
        assert_eq!(
            split_glob_at_first_metachar(r"\\server\share\**\*.rs", &['\\', '/'], true),
            (r"\\server\share\", r"**\*.rs")
        );
        assert_eq!(
            split_glob_at_first_metachar(r"\\?\C:\Users\**\*.rs", &['\\', '/'], true),
            (r"\\?\C:\Users\", r"**\*.rs")
        );
    }

    fn init_test(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let settings_store = SettingsStore::test(cx);
            cx.set_global(settings_store);
        });
    }
}
