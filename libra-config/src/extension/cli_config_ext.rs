use super::global_config_ext::GlobalConfigExt;
use anyhow::{anyhow, bail, Result};
use std::path::PathBuf;
use zapatos::{
    common::{
        types::{
            CliConfig, CliError, CliTypedResult, ConfigSearchMode, ProfileConfig, DEFAULT_PROFILE,
        },
        utils::{create_dir_if_not_exist, read_from_file, write_to_user_only_file},
    },
    config::GlobalConfig,
    genesis::git::from_yaml,
};

const CONFIG_FILE: &str = "config.yaml";
const LEGACY_CONFIG_FILE: &str = "config.yml";

pub trait CliConfigExt {
    fn config_exists_ext(mode: ConfigSearchMode) -> bool;
    fn load_ext(mode: ConfigSearchMode) -> CliTypedResult<CliConfig>;
    fn load_profile_ext(
        profile: Option<&str>,
        mode: ConfigSearchMode,
    ) -> Result<Option<ProfileConfig>>;
    fn save_ext(&self) -> CliTypedResult<()>;
}

impl CliConfigExt for CliConfig {
    fn config_exists_ext(mode: ConfigSearchMode) -> bool {
        if let Ok(folder) = _0l_folder(mode) {
            let config_file = folder.join(CONFIG_FILE);
            let old_config_file = folder.join(LEGACY_CONFIG_FILE);
            config_file.exists() || old_config_file.exists()
        } else {
            false
        }
    }

    /// Loads the config from the current working directory or one of its parents.
    fn load_ext(mode: ConfigSearchMode) -> CliTypedResult<CliConfig> {
        let folder = _0l_folder(mode)?;

        let config_file = folder.join(CONFIG_FILE);
        let old_config_file = folder.join(LEGACY_CONFIG_FILE);
        if config_file.exists() {
            from_yaml(
                &String::from_utf8(read_from_file(config_file.as_path())?)
                    .map_err(CliError::from)?,
            )
        } else if old_config_file.exists() {
            from_yaml(
                &String::from_utf8(read_from_file(old_config_file.as_path())?)
                    .map_err(CliError::from)?,
            )
        } else {
            Err(CliError::ConfigNotFoundError(format!(
                "{}",
                config_file.display()
            )))
        }
    }

    fn load_profile_ext(
        profile: Option<&str>,
        mode: ConfigSearchMode,
    ) -> Result<Option<ProfileConfig>> {
        let config = CliConfig::load_ext(mode);
        if let Some(CliError::ConfigNotFoundError(path)) = config.as_ref().err() {
            bail!("Unable to find config {path}, have you run `libra-config init`?");
        }

        let mut config = config?;
        // If no profile was given, use `default`
        if let Some(profile) = profile {
            if let Some(account_profile) = config.remove_profile(profile) {
                Ok(Some(account_profile))
            } else {
                Err(anyhow!("Profile {} not found", profile))
            }
        } else {
            Ok(config.remove_profile(DEFAULT_PROFILE))
        }
    }

    /// Saves the config to ./.0L/config.yaml
    fn save_ext(&self) -> CliTypedResult<()> {
        let _0l_folder = _0l_folder(ConfigSearchMode::CurrentDir)?;

        // Create if it doesn't exist
        create_dir_if_not_exist(_0l_folder.as_path())?;

        // Save over previous config file
        let config_file = _0l_folder.join(CONFIG_FILE);
        let config_bytes = serde_yaml::to_string(self).map_err(|err| {
            CliError::UnexpectedError(format!("Failed to serialize config {}", err))
        })?;
        write_to_user_only_file(&config_file, CONFIG_FILE, config_bytes.as_bytes())?;

        // As a cleanup, delete the old if it exists
        let legacy_config_file = _0l_folder.join(LEGACY_CONFIG_FILE);
        if legacy_config_file.exists() {
            eprintln!("Removing legacy config file {}", LEGACY_CONFIG_FILE);
            let _ = std::fs::remove_file(legacy_config_file);
        }
        Ok(())
    }
}

fn _0l_folder(mode: ConfigSearchMode) -> CliTypedResult<PathBuf> {
    let global_config = GlobalConfig::load_ext()?;
    global_config.get_config_location_ext(mode)
}
