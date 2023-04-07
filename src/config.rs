use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::{env, fs};

pub struct Config {
    pub node_url: String,
}

impl Config {
    fn from(toml_path: &str) -> Self {
        // First, we try to get the node url from environment variable
        let node_url = env::var("NODE_URL").unwrap_or_else(|_| {
            // If the environment variable is not set, we get the url from toml file
            match Self::read_toml_file_from(toml_path) {
                Ok(toml) => toml.get_node_url(),
                Err(_) => String::from("http://0.0.0.0:8080/"),
            }
        });

        Config { node_url }
    }

    fn read_toml_file_from(path: &str) -> Result<Toml> {
        let bytes = fs::read(path).context(format!("Failed to read 0L.toml file at {path}"))?;
        let str = String::from_utf8(bytes)?;
        toml_edit::de::from_str(&str).context("Unexpected content of 0L.toml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from("~/.0L/0L.toml")
    }
}

#[derive(Deserialize)]
struct Toml {
    profile: Option<Profile>,
}

#[derive(Deserialize)]
struct Profile {
    upstream_nodes: Option<Vec<String>>,
}

impl Toml {
    fn get_node_url(&self) -> String {
        match &self.profile {
            Some(profile) => {
                if let Some(upstream_nodes) = &profile.upstream_nodes {
                    upstream_nodes
                        .choose(&mut rand::thread_rng())
                        .cloned()
                        .unwrap_or_default()
                    //TODO: make sure that the selected url is alive
                } else {
                    String::new()
                }
            }
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::Config;

    #[test]
    fn get_configs_correctly() {
        let result = Config::from("tests/0L.toml");
        let upstream_nodes = [
            "http://localhost:8080/",
            "http://104.248.94.195:8080/",
            "http://63.229.234.76:8080/",
        ];
        assert!(upstream_nodes.contains(&result.node_url.as_str()));

        env::set_var("NODE_URL", "test-url");
        assert_eq!("test-url", Config::default().node_url);

        env::remove_var("NODE_URL");
        assert_eq!(
            "http://0.0.0.0:8080/",
            Config::from("invalid_toml_path").node_url
        );
    }
}