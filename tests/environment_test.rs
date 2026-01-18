//! Tests for environment variable expansion functionality

#[cfg(test)]
mod tests {
    use agterm::config::EnvironmentConfig;
    use agterm::terminal::pty::PtyEnvironment;
    use std::collections::HashMap;

    #[test]
    fn test_environment_config_default() {
        let env_config = EnvironmentConfig::default();
        assert_eq!(env_config.inherit, true);
        assert_eq!(env_config.term, "xterm-256color");
        assert!(env_config.variables.is_empty());
        assert!(env_config.lang.is_none());
        assert!(env_config.path_prepend.is_empty());
        assert!(env_config.path_append.is_empty());
    }

    #[test]
    fn test_to_pty_environment_basic() {
        let env_config = EnvironmentConfig::default();
        let pty_env = env_config.to_pty_environment();

        assert_eq!(pty_env.inherit_env, true);
        assert_eq!(pty_env.variables.get("TERM"), Some(&"xterm-256color".to_string()));
        assert_eq!(pty_env.variables.get("COLORTERM"), Some(&"truecolor".to_string()));
        assert_eq!(pty_env.variables.get("TERM_PROGRAM"), Some(&"agterm".to_string()));
        assert!(pty_env.variables.contains_key("AGTERM_VERSION"));
    }

    #[test]
    fn test_to_pty_environment_with_custom_variables() {
        let mut env_config = EnvironmentConfig::default();
        env_config.variables.insert("MY_VAR".to_string(), "test_value".to_string());
        env_config.variables.insert("ANOTHER_VAR".to_string(), "123".to_string());

        let pty_env = env_config.to_pty_environment();

        assert_eq!(pty_env.variables.get("MY_VAR"), Some(&"test_value".to_string()));
        assert_eq!(pty_env.variables.get("ANOTHER_VAR"), Some(&"123".to_string()));
    }

    #[test]
    fn test_to_pty_environment_with_lang() {
        let mut env_config = EnvironmentConfig::default();
        env_config.lang = Some("en_US.UTF-8".to_string());

        let pty_env = env_config.to_pty_environment();

        assert_eq!(pty_env.variables.get("LANG"), Some(&"en_US.UTF-8".to_string()));
    }

    #[test]
    fn test_to_pty_environment_path_prepend() {
        let mut env_config = EnvironmentConfig::default();
        env_config.path_prepend.push("/usr/local/bin".to_string());
        env_config.path_prepend.push("/opt/bin".to_string());

        let pty_env = env_config.to_pty_environment();

        let path = pty_env.variables.get("PATH").expect("PATH should be set");
        assert!(path.starts_with("/usr/local/bin:/opt/bin:"));
    }

    #[test]
    fn test_to_pty_environment_path_append() {
        let mut env_config = EnvironmentConfig::default();
        env_config.path_append.push("$HOME/.local/bin".to_string());

        let pty_env = env_config.to_pty_environment();

        let path = pty_env.variables.get("PATH").expect("PATH should be set");
        assert!(path.ends_with(":$HOME/.local/bin"));
    }

    #[test]
    fn test_to_pty_environment_override_defaults() {
        let mut env_config = EnvironmentConfig::default();
        env_config.variables.insert("TERM".to_string(), "xterm-kitty".to_string());
        env_config.variables.insert("COLORTERM".to_string(), "24bit".to_string());

        let pty_env = env_config.to_pty_environment();

        // User variables should override defaults
        assert_eq!(pty_env.variables.get("TERM"), Some(&"xterm-kitty".to_string()));
        assert_eq!(pty_env.variables.get("COLORTERM"), Some(&"24bit".to_string()));
    }

    #[test]
    fn test_environment_config_no_inherit() {
        let mut env_config = EnvironmentConfig::default();
        env_config.inherit = false;

        let pty_env = env_config.to_pty_environment();

        assert_eq!(pty_env.inherit_env, false);
    }

    #[test]
    fn test_pty_environment_creation() {
        let mut variables = HashMap::new();
        variables.insert("TEST_VAR".to_string(), "test_value".to_string());

        let pty_env = PtyEnvironment {
            inherit_env: true,
            variables: variables.clone(),
            unset: vec!["OLD_VAR".to_string()],
        };

        assert_eq!(pty_env.inherit_env, true);
        assert_eq!(pty_env.variables.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(pty_env.unset.len(), 1);
        assert_eq!(pty_env.unset[0], "OLD_VAR");
    }
}
