use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: String,
    pub provider: String,
    pub session_dir: PathBuf,
    pub max_steps: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            model: "mock".to_string(),
            provider: "mock".to_string(),
            session_dir: PathBuf::from("data/sessions"),
            max_steps: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PartialConfig {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub session_dir: Option<PathBuf>,
    pub max_steps: Option<usize>,
}

impl Config {
    #[allow(dead_code)]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let partial: PartialConfig = serde_json::from_str(&content)?;
        let default = Config::default();
        Ok(Config {
            model: partial.model.unwrap_or(default.model),
            provider: partial.provider.unwrap_or(default.provider),
            session_dir: partial.session_dir.unwrap_or(default.session_dir),
            max_steps: partial.max_steps.unwrap_or(default.max_steps),
        })
    }

    pub fn from_env() -> Self {
        let default = Config::default();
        let model = std::env::var("SWAL_MODEL").unwrap_or(default.model);
        let provider = std::env::var("SWAL_PROVIDER").unwrap_or(default.provider);
        let session_dir = std::env::var("SWAL_SESSION_DIR")
            .map(PathBuf::from)
            .unwrap_or(default.session_dir);
        let max_steps = std::env::var("SWAL_MAX_STEPS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(default.max_steps);

        Config {
            model,
            provider,
            session_dir,
            max_steps,
        }
    }

    pub fn load(args_config: Option<PathBuf>) -> anyhow::Result<Self> {
        let env_config = Self::from_env();
        if let Some(path) = args_config {
            let content = std::fs::read_to_string(path)?;
            let partial: PartialConfig = serde_json::from_str(&content)?;
            Ok(Config {
                model: partial.model.unwrap_or(env_config.model),
                provider: partial.provider.unwrap_or(env_config.provider),
                session_dir: partial.session_dir.unwrap_or(env_config.session_dir),
                max_steps: partial.max_steps.unwrap_or(env_config.max_steps),
            })
        } else {
            Ok(env_config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.model, "mock");
        assert_eq!(config.provider, "mock");
        assert_eq!(config.session_dir.to_str().unwrap(), "data/sessions");
        assert_eq!(config.max_steps, 10);
    }

    #[test]
    fn test_config_from_file_complete() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("swal_config_complete.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        use std::io::Write;
        writeln!(
            file,
            r#"{{"model": "gpt-4", "provider": "openai", "session_dir": "custom/path", "max_steps": 25}}"#
        )
        .unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.session_dir.to_str().unwrap(), "custom/path");
        assert_eq!(config.max_steps, 25);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_config_from_file_partial() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("swal_config_partial.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        use std::io::Write;
        writeln!(file, r#"{{"model": "claude-3"}}"#).unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert_eq!(config.model, "claude-3");
        assert_eq!(config.provider, "mock");
        assert_eq!(config.session_dir.to_str().unwrap(), "data/sessions");
        assert_eq!(config.max_steps, 10);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_config_load_precedence() {
        std::env::set_var("SWAL_MODEL", "env-model");
        std::env::set_var("SWAL_PROVIDER", "env-provider");
        std::env::set_var("SWAL_MAX_STEPS", "42");

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("swal_config_precedence.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        use std::io::Write;
        writeln!(file, r#"{{"model": "file-model"}}"#).unwrap();

        let config = Config::load(Some(file_path.clone())).unwrap();
        assert_eq!(config.model, "file-model"); // file overrides env
        assert_eq!(config.provider, "env-provider"); // env fallback
        assert_eq!(config.max_steps, 42); // env fallback
        assert_eq!(config.session_dir.to_str().unwrap(), "data/sessions"); // default fallback

        std::env::remove_var("SWAL_MODEL");
        std::env::remove_var("SWAL_PROVIDER");
        std::env::remove_var("SWAL_MAX_STEPS");
        let _ = std::fs::remove_file(file_path);
    }
}
