use anyhow::Result;
use serde_json::Value;

use crate::migrations::migrate_settings;

const AGENT_KEY: &str = "agent";
const DEFAULT_KEY: &str = "default";
const DEFAULT_MODE_KEY: &str = "default_mode";
const PERMISSION_MODE_KEY: &str = "permission_mode";
const TOOL_PERMISSIONS_KEY: &str = "tool_permissions";

pub fn migrate_agent_permission_modes(value: &mut Value) -> Result<()> {
    migrate_settings(value, &mut migrate_one)
}

fn migrate_one(settings: &mut serde_json::Map<String, Value>) -> Result<()> {
    let Some(Value::Object(agent)) = settings.get_mut(AGENT_KEY) else {
        return Ok(());
    };

    let Some(Value::Object(tool_permissions)) = agent.get_mut(TOOL_PERMISSIONS_KEY) else {
        return Ok(());
    };

    let legacy_key = if tool_permissions.contains_key(DEFAULT_KEY) {
        DEFAULT_KEY
    } else if tool_permissions.contains_key(DEFAULT_MODE_KEY) {
        DEFAULT_MODE_KEY
    } else {
        return Ok(());
    };
    let Some(legacy_default) = tool_permissions.remove(legacy_key) else {
        return Ok(());
    };

    let permission_mode = match legacy_default.as_str() {
        Some("allow") => Some("auto"),
        Some("confirm") => Some("manual"),
        // The new model has no global deny-all mode. Manual is the safest
        // interactive replacement because unmatched actions still cannot run
        // without explicit approval.
        Some("deny") => Some("manual"),
        None if legacy_default.is_null() => None,
        _ => {
            tool_permissions.insert(legacy_key.to_string(), legacy_default);
            return Ok(());
        }
    };
    let remove_tool_permissions = tool_permissions.is_empty();

    if remove_tool_permissions {
        agent.remove(TOOL_PERMISSIONS_KEY);
    }

    let permission_mode_is_missing = !matches!(
        agent.get(PERMISSION_MODE_KEY),
        Some(permission_mode) if !permission_mode.is_null()
    );
    if permission_mode_is_missing && let Some(permission_mode) = permission_mode {
        agent.insert(
            PERMISSION_MODE_KEY.to_string(),
            Value::String(permission_mode.to_string()),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn migrate(mut value: Value) -> Value {
        migrate_agent_permission_modes(&mut value).expect("permission migration should succeed");
        value
    }

    #[test]
    fn migrates_global_defaults_to_permission_modes() {
        assert_eq!(
            migrate(json!({
                "agent": {
                    "tool_permissions": { "default": "allow" }
                }
            })),
            json!({
                "agent": {
                    "permission_mode": "auto"
                }
            })
        );
        assert_eq!(
            migrate(json!({
                "agent": {
                    "tool_permissions": { "default": "confirm" }
                }
            })),
            json!({
                "agent": {
                    "permission_mode": "manual"
                }
            })
        );
        assert_eq!(
            migrate(json!({
                "agent": {
                    "tool_permissions": { "default": "deny" }
                }
            })),
            json!({
                "agent": {
                    "permission_mode": "manual"
                }
            })
        );
    }

    #[test]
    fn preserves_tool_rules_and_explicit_permission_mode() {
        assert_eq!(
            migrate(json!({
                "agent": {
                    "permission_mode": "full_access",
                    "tool_permissions": {
                        "default": "confirm",
                        "tools": {
                            "terminal": {
                                "always_deny": [{ "pattern": "rm -rf" }]
                            }
                        }
                    }
                }
            })),
            json!({
                "agent": {
                    "permission_mode": "full_access",
                    "tool_permissions": {
                        "tools": {
                            "terminal": {
                                "always_deny": [{ "pattern": "rm -rf" }]
                            }
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn migrates_platform_and_settings_profile_overrides() {
        assert_eq!(
            migrate(json!({
                "macos": {
                    "agent": {
                        "tool_permissions": { "default": "allow" }
                    }
                },
                "profiles": {
                    "safe": {
                        "settings": {
                            "agent": {
                                "tool_permissions": { "default": "confirm" }
                            }
                        }
                    }
                }
            })),
            json!({
                "macos": {
                    "agent": {
                        "permission_mode": "auto"
                    }
                },
                "profiles": {
                    "safe": {
                        "settings": {
                            "agent": {
                                "permission_mode": "manual"
                            }
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn migrates_the_original_boolean_permission_setting_through_both_steps() {
        let mut value = json!({
            "agent": {
                "always_allow_tool_actions": true
            }
        });
        crate::migrations::m_2026_02_04::migrate_tool_permission_defaults(&mut value)
            .expect("legacy boolean migration should succeed");
        migrate_agent_permission_modes(&mut value)
            .expect("permission mode migration should succeed");

        assert_eq!(
            value,
            json!({
                "agent": {
                    "permission_mode": "auto"
                }
            })
        );
    }

    #[test]
    fn leaves_invalid_legacy_defaults_for_validation() {
        let input = json!({
            "agent": {
                "tool_permissions": { "default": "sometimes" }
            }
        });
        assert_eq!(migrate(input.clone()), input);
    }
}
