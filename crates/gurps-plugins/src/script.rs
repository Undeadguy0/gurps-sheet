//! Scripting interface (stub). A Rhai-backed implementation is planned behind
//! the `rhai` cargo feature; for now only the interface and a no-op engine exist.

use gurps_core::Character;

/// Result of evaluating a script hook.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScriptOutput {
    pub messages: Vec<String>,
}

/// A scripting backend that plugins may use for rules not expressible as data.
pub trait ScriptEngine {
    fn name(&self) -> &str;
    /// Evaluates `source` against a character (read-only for now).
    fn eval(&self, source: &str, character: &Character) -> Result<ScriptOutput, String>;
}

/// Engine that refuses to run anything.
#[derive(Debug, Default)]
pub struct NoScripting;

impl ScriptEngine for NoScripting {
    fn name(&self) -> &str {
        "none"
    }
    fn eval(&self, _source: &str, _character: &Character) -> Result<ScriptOutput, String> {
        Err("scripting is not enabled in this build".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_engine_rejects() {
        assert!(NoScripting.eval("1+1", &Character::default()).is_err());
    }
}
