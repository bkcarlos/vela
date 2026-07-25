pub use env_var::{EnvVar, bool_env_var, env_var};
use std::sync::LazyLock;

/// Whether Vela is running in stateless mode.
/// When true, Vela will use in-memory databases instead of persistent storage.
pub static VELA_STATELESS: LazyLock<bool> = bool_env_var!("VELA_STATELESS");
