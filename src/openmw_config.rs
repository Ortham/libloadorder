/*
Notes on OpenMW support:

OpenMW defines its load order and its external plugins directories in its config
files. Unfortunately, where those config files are loaded from is a mix of
platform-specific, configured at compile time, and configured within other
config files. The last config file to be loaded is the one that gets treated as
writeable user config. See here for reference:

<https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/configurationmanager.cpp>
<https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/linuxpath.cpp>
<https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/windowspath.cpp>

This is further complicated when OpenMW is installed as a Flatpak app, because
the config may define paths that are only valid within the app's Flatpak runtime
environment (e.g. starting with /app). It's also possible to define config at
runtime using CLI parameters.

libloadorder attempts to support this system, minus the CLI parameters. The game
path given when creating a GameSettings object must be the path containing the
OpenMW executable.

If a local path is provided, it is used to replace the last config file that is
loaded with the config in the local path. If there is no config in the local
path, the config from last loaded config file is effectively removed. If
provided, any `config` entries in the local path's config file will be ignored.

On Linux, OpenMW's global config and global data paths are specified at compile
time, and I'm not aware of any way to retrieve them. As such, libloadorder
guesses their paths based on the game path, so that the correct paths should be
guessed for all of the release distributions listed at
<https://openmw.org/downloads/>.
*/
#[cfg(not(windows))]
use std::ffi::OsString;
use std::{
    collections::HashSet,
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use crate::Error;

pub fn user_config_dir(game_path: &Path) -> Result<PathBuf, Error> {
    let fixed_paths = FixedPaths::new(game_path)?;
    let config_state = load_game_configs(&fixed_paths)?;

    Ok(config_state.user_config_dir)
}

pub fn resources_vfs_path(game_path: &Path, local_path: &Path) -> Result<PathBuf, Error> {
    let config = load_game_config_with_user_config_dir(game_path, local_path)?;

    // Default value is relative to OpenMW's current working directory, assume
    // that's the OpenMW executable's directory, i.e. the game path.
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/config/gamesettings.cpp?ref_type=tags#L61>
    Ok(config
        .resources
        .unwrap_or_else(|| game_path.join("resources"))
        .join("vfs"))
}

pub fn additional_data_paths(game_path: &Path, local_path: &Path) -> Result<Vec<PathBuf>, Error> {
    load_game_config_with_user_config_dir(game_path, local_path)
        .map(|c| c.into_additional_data_paths())
}

pub fn non_user_additional_data_paths(game_path: &Path) -> Result<Vec<PathBuf>, Error> {
    load_non_user_config(game_path).map(|c| c.into_additional_data_paths())
}

pub fn read_active_plugin_names(user_config_path: &Path) -> Result<Vec<String>, Error> {
    let ini = match read_openmw_cfg(user_config_path)? {
        Some(ini) => ini,
        None => return Ok(Vec::new()),
    };

    let active_plugin_names: Vec<_> = ini
        .general_section()
        .get_all("content")
        .map(|v| v.to_string())
        .collect();

    Ok(active_plugin_names)
}

pub fn non_user_active_plugin_names(game_path: &Path) -> Result<Vec<String>, Error> {
    load_non_user_config(game_path).map(|c| c.content)
}

#[cfg(windows)]
fn default_user_config_dir() -> Result<PathBuf, Error> {
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/windowspath.cpp?ref_type=tags#L35>
    dirs::document_dir()
        .map(|d| d.join("My Games\\OpenMW"))
        .ok_or_else(|| Error::NoDocumentsPath)
}

#[cfg(not(windows))]
fn default_user_config_dir(game_path: &Path) -> Result<PathBuf, Error> {
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/linuxpath.cpp?ref_type=tags#L64>
    if is_flatpak_install(game_path) {
        // When run as a Flatpak, OpenMW sees a different XDG_CONFIG_HOME
        // value.
        dirs::home_dir().map(|d| d.join(".var/app/org.openmw.OpenMW/config/openmw"))
    } else {
        // Checking $HOST_XDG_CONFIG_HOME first in case libloadorder is running
        // as part of a Flatpak app.
        std::env::var_os("HOST_XDG_CONFIG_HOME")
            .and_then(is_absolute_path)
            .or_else(|| dirs::config_dir())
            .map(|p| p.join("openmw"))
    }
    .ok_or_else(|| Error::NoUserConfigPath)
}

#[cfg(not(windows))]
fn default_user_data_dir(is_flatpak_install: bool) -> Result<PathBuf, Error> {
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/linuxpath.cpp?ref_type=tags#L69>
    if is_flatpak_install {
        dirs::home_dir().map(|d| d.join(".var/app/org.openmw.OpenMW/data/openmw"))
    } else {
        // Checking $HOST_XDG_DATA_HOME first in case libloadorder is
        // running as part of a Flatpak app.
        std::env::var_os("HOST_XDG_DATA_HOME")
            .and_then(is_absolute_path)
            .or_else(|| dirs::data_local_dir())
            .map(|p| p.join("openmw"))
    }
    .ok_or_else(|| Error::NoUserDataPath)
}

#[cfg(windows)]
fn default_global_config_dir() -> Result<PathBuf, Error> {
    // This is similar to OpenMW's approach here:
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/windowspath.cpp?ref_type=tags#L55>
    // Though it errors instead of falling back to the current working directory
    // if the Program Files path cannot be obtained.
    use std::{
        ffi::{c_void, OsString},
        os::windows::ffi::OsStringExt,
    };
    use windows::Win32::UI::Shell;

    // Unfortunately there's no safe API wrapper for this.
    unsafe {
        // There's nothing unsafe about calling this function with these
        // arguments, but care is needed with the returned PWSTR.
        let pwstr = Shell::SHGetKnownFolderPath(
            &Shell::FOLDERID_ProgramFiles,
            Shell::KNOWN_FOLDER_FLAG(0),
            None,
        )?;

        // It's not safe to call .as_wide() on a null PWSTR.
        if pwstr.is_null() {
            return Err(Error::NoProgramFilesPath);
        }

        let program_files_path = PathBuf::from(OsString::from_wide(pwstr.as_wide()));

        // Now free the pwstr as documented here:
        // <https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath>
        windows::Win32::System::Com::CoTaskMemFree(Some(pwstr.as_ptr() as *const c_void));

        Ok(program_files_path.join("OpenMW"))
    }
}

#[cfg(not(windows))]
fn default_global_config_dir(game_path: &Path) -> PathBuf {
    // This path is hardcoded at compile time:
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/linuxpath.cpp?ref_type=tags#L81>
    // So just make an educated guess based on the game path.
    if game_path.join("resources/vfs").exists() {
        // Probably a single directory install from the release archive.
        game_path.to_path_buf()
    } else if is_flatpak_install(game_path) {
        // Probably installed from Flathub.
        game_path.join("../etc/openmw")
    } else {
        // Probably installed from the OS package manager.
        let host_etc_path = Path::new("/run/host/etc/openmw");
        if host_etc_path.exists() {
            // libloadorder is probably running as part of a Flatpak app.
            host_etc_path.into()
        } else {
            "/etc/openmw".into()
        }
    }
}

#[cfg(not(windows))]
fn default_global_data_dir(game_path: &Path) -> PathBuf {
    // This path is hardcoded at compile time:
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/linuxpath.cpp?ref_type=tags#L108>
    // So just make an educated guess based on the game path.
    if game_path.join("resources/vfs").exists() {
        // Probably a single directory install from the release archive.
        game_path.to_path_buf()
    } else if is_flatpak_install(game_path) {
        // Probably installed from Flathub.
        game_path.join("../share/games/openmw")
    } else {
        let host_share_games_path = Path::new("/run/host/usr/share/games/openmw");
        let host_share_path = Path::new("/run/host/usr/share/openmw");
        let share_games_path = Path::new("/usr/share/games/openmw");
        if host_share_games_path.exists() {
            // libloadorder is probably running as part of a Flatpak app, and
            // OpenMW was probably installed using Ubuntu's, Debian's or Arch's
            // package manager.
            host_share_games_path.into()
        } else if host_share_path.exists() {
            // libloadorder is probably running as part of a Flatpak app, and
            // OpenMW was probably installed using OpenSUSE's package manager.
            host_share_path.into()
        } else if share_games_path.exists() {
            // Probably installed using Ubuntu's, Debian's or Arch's package
            // manager.
            share_games_path.into()
        } else {
            // Probably installed using OpenSUSE's package manager.
            "/usr/share/openmw".into()
        }
    }
}

#[cfg(not(windows))]
fn is_absolute_path(value: OsString) -> Option<PathBuf> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Some(path)
    } else {
        None
    }
}

#[cfg(not(windows))]
fn is_flatpak_install(game_path: &Path) -> bool {
    // The presence of a metadata file seems to be the most reliable indicator
    // of a Flatpak install.
    // <https://docs.flatpak.org/en/latest/flatpak-command-reference.html#flatpak-metadata>
    // The game path is expected to be files/share/games/openmw, relative to the
    // Flatpak app's top-level deploy directory (where the metadata file is).
    let metadata_file_path = game_path.join("../../metadata");

    ini::Ini::load_from_file(metadata_file_path)
        .map(|ini| {
            if let Some(name) = ini.get_from(Some("Application"), "name") {
                name == "org.openmw.OpenMW"
            } else {
                false
            }
        })
        .unwrap_or(false)
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct FixedPaths {
    local: PathBuf,
    user_config: PathBuf,
    user_data: PathBuf,
    global_config: PathBuf,
    global_data: PathBuf,
    flatpak_app: Option<PathBuf>,
}

impl FixedPaths {
    #[cfg(windows)]
    fn new(game_path: &Path) -> Result<FixedPaths, Error> {
        let user_config = default_user_config_dir()?;
        let global_config = default_global_config_dir()?;

        // The user data path is the same as the user config path on Windows:
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/windowspath.cpp?ref_type=tags#L49>
        let user_data = user_config.clone();

        // The global data path is the same as the global config path on Windows:
        // // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/windowspath.cpp?ref_type=tags#L84>
        let global_data = global_config.clone();

        Ok(FixedPaths {
            local: game_path.to_path_buf(),
            user_config,
            user_data,
            global_config,
            global_data,
            flatpak_app: None,
        })
    }

    #[cfg(not(windows))]
    fn new(game_path: &Path) -> Result<FixedPaths, Error> {
        let is_flatpak_install = is_flatpak_install(game_path);

        let user_config = default_user_config_dir(game_path)?;
        let user_data = default_user_data_dir(is_flatpak_install)?;
        let global_config = default_global_config_dir(game_path);
        let global_data = default_global_data_dir(game_path);

        Ok(FixedPaths {
            local: game_path.to_path_buf(),
            user_config,
            user_data,
            global_config,
            global_data,
            flatpak_app: is_flatpak_install.then(|| game_path.join("..")),
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct OpenMWConfig {
    replace: Vec<String>,
    config: Vec<PathBuf>,
    resources: Option<PathBuf>,
    data_local: Option<PathBuf>,
    data: Vec<PathBuf>,
    content: Vec<String>,
}

impl OpenMWConfig {
    fn reduce(configs: Vec<Self>) -> Self {
        configs
            .into_iter()
            .reduce(|acc, e| e.reduce_into(acc))
            .unwrap_or_default()
    }

    fn reduce_into(mut self, mut accumulator: Self) -> Self {
        OpenMWConfig {
            resources: accumulator.resources.or(self.resources),
            data_local: accumulator.data_local.or(self.data_local),
            config: if self.replace.iter().any(|r| r == "config") {
                self.config
            } else {
                accumulator.config.append(&mut self.config);
                accumulator.config
            },
            data: if self.replace.iter().any(|r| r == "data") {
                self.data
            } else {
                accumulator.data.append(&mut self.data);
                accumulator.data
            },
            content: if self.replace.iter().any(|r| r == "content") {
                self.content
            } else {
                accumulator.content.append(&mut self.content);
                accumulator.content
            },
            replace: if self.replace.iter().any(|r| r == "replace") {
                self.replace
            } else {
                accumulator.replace.append(&mut self.replace);
                accumulator.replace
            },
        }
    }

    // This includes the value of data-local and the data values, but not the value of
    // <resources>/vfs. The value of user-data is of no interest to libloadorder.
    fn into_additional_data_paths(self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(path) = self.data_local {
            paths.push(path);
        }

        paths.extend(self.data);

        paths
    }
}

fn parse_path_value(value: &str) -> String {
    // Values may be enclosed in double quotes and use & as an escape, see:
    // <https://github.com/OpenMW/openmw/blob/openmw-0.48.0/components/config/gamesettings.cpp#L124>
    if !value.starts_with("\"") {
        return value.to_string();
    }

    // Although the cfg file is encoded in UTF-8, OpenMW iterates over UTF-16
    // code points. This iterates over Unicode scalar values: the only
    // difference is the absence of surrogate code points in the latter, which
    // is not a problem because they're only a representational artifact of
    // UTF-16, so the result will be the same in both cases.
    let mut result = String::with_capacity(value.len() - 1);
    let mut chars = value[1..].chars();

    while let Some(c) = chars.next() {
        if c == '"' {
            break;
        }

        if c == '&' {
            if let Some(c) = chars.next() {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn resolve_path_value(
    parsed_path: String,
    config_dir_path: &Path,
    fixed_paths: &FixedPaths,
) -> Option<PathBuf> {
    if parsed_path.is_empty() || !parsed_path.starts_with('?') {
        if Path::new(&parsed_path).is_relative() {
            Some(config_dir_path.join(parsed_path))
        } else if let Some(app_path) = &fixed_paths.flatpak_app {
            // Paths within the Flatpak app may be relative to the app's runtime
            // mount point, so replace that with the path of the Flatpak app's
            // files at rest on the host.
            if let Some(suffix) = parsed_path.strip_prefix("/app/") {
                Some(app_path.join(suffix))
            } else {
                Some(parsed_path.into())
            }
        } else {
            Some(parsed_path.into())
        }
    } else if let Some((token, suffix)) = parsed_path[1..].split_once('?') {
        let token_path = match token {
            "local" => &fixed_paths.local,
            "userconfig" => &fixed_paths.user_config,
            "userdata" => &fixed_paths.user_data,
            "global" => &fixed_paths.global_data,
            _ => return None,
        };
        if suffix.is_empty() {
            Some(token_path.clone())
        } else {
            Some(token_path.join(suffix))
        }
    } else {
        Some(parsed_path.into())
    }
}

fn load_config(
    config_dir_path: &Path,
    fixed_paths: &FixedPaths,
) -> Result<Option<OpenMWConfig>, Error> {
    let ini = match read_openmw_cfg(&config_dir_path.join("openmw.cfg"))? {
        Some(ini) => ini,
        None => return Ok(None),
    };

    let path_mapper = |s| resolve_path_value(parse_path_value(s), config_dir_path, fixed_paths);

    let data: Vec<_> = ini
        .general_section()
        .get_all("data")
        .filter_map(path_mapper)
        .collect();

    let resources = ini.general_section().get("resources").and_then(path_mapper);

    let data_local = ini
        .general_section()
        .get("data-local")
        .and_then(path_mapper);

    let config: Vec<_> = ini
        .general_section()
        .get_all("config")
        .filter_map(path_mapper)
        .collect();

    let replace: Vec<_> = ini
        .general_section()
        .get_all("replace")
        .map(|s| s.to_string())
        .collect();

    let content: Vec<_> = ini
        .general_section()
        .get_all("content")
        .map(|v| v.to_string())
        .collect();

    Ok(Some(OpenMWConfig {
        config,
        replace,
        resources,
        data,
        data_local,
        content,
    }))
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct OpenMWConfigState {
    loaded_configs: Vec<OpenMWConfig>,
    user_config_dir: PathBuf,
}

fn load_game_configs(fixed_paths: &FixedPaths) -> Result<OpenMWConfigState, Error> {
    // This is based on
    // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc4/components/files/configurationmanager.cpp#L65>
    // but skips handling of config provided as CLI parameters.

    let mut active_config_paths = Vec::new();

    let mut config = load_config(&fixed_paths.local, fixed_paths)?;
    if config.is_some() {
        active_config_paths.push(fixed_paths.local.clone());
    } else {
        active_config_paths.push(fixed_paths.global_config.clone());
        config = load_config(&fixed_paths.global_config, fixed_paths)?;
    }

    let config = match config {
        Some(c) => c,
        None => {
            return Ok(OpenMWConfigState {
                loaded_configs: Vec::new(),
                user_config_dir: fixed_paths.global_config.clone(),
            })
        }
    };

    let mut already_parsed_paths = HashSet::new();
    for path in &active_config_paths {
        already_parsed_paths.insert(path.clone());
    }

    let mut extra_config_dirs = config.config.clone();

    let mut parsed_configs = vec![config];

    while let Some(path) = extra_config_dirs.pop() {
        if already_parsed_paths.contains(&path) {
            continue;
        }

        already_parsed_paths.insert(path.clone());

        if let Some(config) = load_config(&path, fixed_paths)? {
            if config.replace.iter().any(|r| r == "config") && parsed_configs.len() > 1 {
                active_config_paths.truncate(1);
                parsed_configs.truncate(1);
            }

            extra_config_dirs.extend_from_slice(&config.config);
            parsed_configs.push(config);
        } else {
            // Record an empty config so that if this is the last (i.e. user)
            // config, we can pop it off to get the non-user configs.
            parsed_configs.push(OpenMWConfig::default())
        }

        active_config_paths.push(path);
    }

    Ok(OpenMWConfigState {
        loaded_configs: parsed_configs,
        user_config_dir: active_config_paths
            .last()
            .expect("There is at least one loaded config")
            .clone(),
    })
}

fn load_game_config_with_user_config_dir(
    game_path: &Path,
    user_config_dir: &Path,
) -> Result<OpenMWConfig, Error> {
    let fixed_paths = FixedPaths::new(game_path)?;
    let mut config_state = load_game_configs(&fixed_paths)?;

    if config_state.user_config_dir != user_config_dir {
        // Replace the last config with one from the given dir.
        let new_config = load_config(user_config_dir, &fixed_paths)?.unwrap_or_default();

        config_state.loaded_configs.pop();
        config_state.loaded_configs.push(new_config);
    }

    Ok(OpenMWConfig::reduce(config_state.loaded_configs))
}

fn load_non_user_config(game_path: &Path) -> Result<OpenMWConfig, Error> {
    let fixed_paths = FixedPaths::new(game_path)?;
    let mut config_state = load_game_configs(&fixed_paths)?;

    // We don't want the user config, so omit the last config when merging.
    config_state.loaded_configs.pop();

    Ok(OpenMWConfig::reduce(config_state.loaded_configs))
}

fn read_openmw_cfg(openmw_cfg_path: &Path) -> Result<Option<ini::Ini>, Error> {
    if !openmw_cfg_path.exists() {
        return Ok(None);
    }

    // openmw.cfg is encoded in UTF-8, see:
    // <https://github.com/OpenMW/openmw/blob/openmw-0.48.0/components/config/gamesettings.cpp#L237>
    ini::Ini::load_from_file_opt(
        openmw_cfg_path,
        ini::ParseOption {
            enabled_quote: false,
            enabled_escape: false,
        },
    )
    .map(Some)
    .map_err(|e| match e {
        ini::Error::Io(e) => Error::IoError(openmw_cfg_path.to_path_buf(), e),
        ini::Error::Parse(e) => Error::IniParsingError {
            path: openmw_cfg_path.to_path_buf(),
            line: e.line,
            column: e.col,
            message: e.msg.to_string(),
        },
    })
}

fn escape_openmw_data_value(value: &Path) -> Result<String, Error> {
    let str_value = value
        .to_str()
        .ok_or_else(|| Error::InvalidPath(value.to_path_buf()))?;

    let mut result = String::with_capacity(str_value.len() + 2);

    result.push('"');
    for char in str_value.chars() {
        if char == '&' || char == '"' {
            result.push('&');
        }

        result.push(char);
    }
    result.push('"');

    Ok(result)
}

pub fn write_openmw_cfg(
    openmw_cfg_path: &Path,
    data_paths: &[PathBuf],
    active_plugin_names: &[&str],
) -> Result<(), Error> {
    let mut ini = match read_openmw_cfg(openmw_cfg_path)? {
        Some(ini) => ini,
        None => ini::Ini::new(),
    };

    // Remove existing data paths.
    let _ = ini.general_section_mut().remove_all("data").count();

    // Add data paths.
    for data_path in data_paths {
        ini.general_section_mut()
            .append("data", escape_openmw_data_value(data_path)?);
    }

    // Remove existing load order.
    let _ = ini.general_section_mut().remove_all("content").count();

    // Add plugins in load order.
    for plugin_name in active_plugin_names {
        ini.general_section_mut()
            .append("content", plugin_name.to_string());
    }

    if let Some(parent_path) = openmw_cfg_path.parent().filter(|p| !p.exists()) {
        create_dir_all(parent_path).map_err(|e| Error::IoError(parent_path.to_path_buf(), e))?;
    }

    // OpenMW's launcher doesn't escape backslashes.
    ini.write_to_file_policy(openmw_cfg_path, ini::EscapePolicy::Nothing)
        .map_err(|e| Error::IoError(openmw_cfg_path.to_path_buf(), e))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn fixed_paths() -> FixedPaths {
        FixedPaths {
            local: "a".into(),
            user_config: "b".into(),
            user_data: "c".into(),
            global_config: "d".into(),
            global_data: "e".into(),
            flatpak_app: Some("f".into()),
        }
    }

    #[test]
    fn resources_vfs_path_should_be_relative_to_game_path_if_not_defined_in_config() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");

        let path = resources_vfs_path(&game_path, &tmp_dir.path().join("local")).unwrap();

        assert_eq!(game_path.join("resources/vfs"), path);
    }

    #[test]
    fn read_active_plugin_names_should_return_content_values_in_order() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(&ini_path, "content=a\ncontent=b\ncontent=c").unwrap();

        let data_paths = read_active_plugin_names(&ini_path).unwrap();

        let expected_names: &[String] = &["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(expected_names, data_paths);
    }

    #[test]
    fn read_active_plugin_names_should_not_error_if_the_given_path_does_not_exist() {
        let data_paths = read_active_plugin_names(Path::new("missing")).unwrap();

        assert!(data_paths.is_empty());
    }

    #[test]
    #[cfg(windows)]
    fn default_user_config_dir_on_windows_should_be_in_my_games() {
        let path = default_user_config_dir().unwrap();

        assert_eq!(dirs::document_dir().unwrap().join("My Games\\OpenMW"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_user_config_dir_on_linux_flatpak_should_use_user_flatpak_config_dir() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=org.openmw.OpenMW",
        )
        .unwrap();

        let path = default_user_config_dir(&game_path).unwrap();

        assert_eq!(
            dirs::home_dir()
                .unwrap()
                .join(".var/app/org.openmw.OpenMW/config/openmw"),
            path
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn default_user_config_dir_on_linux_non_flatpak_should_use_user_config() {
        // Changing env vars on Linux within a running process is a can of worms, so don't try
        // setting HOST_XDG_CONFIG_HOME.
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let path = default_user_config_dir(game_path).unwrap();

        assert_eq!(dirs::config_dir().unwrap().join("openmw"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_user_data_dir_on_linux_flatpak_should_use_user_flatpak_data_dir() {
        let path = default_user_data_dir(true).unwrap();

        assert_eq!(
            dirs::home_dir()
                .unwrap()
                .join(".var/app/org.openmw.OpenMW/data/openmw"),
            path
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn default_user_data_dir_on_linux_non_flatpak_should_user_local_data() {
        // Changing env vars on Linux within a running process is a can of worms, so don't try
        // setting HOST_XDG_DATA_HOME.
        let path = default_user_data_dir(false).unwrap();

        assert_eq!(dirs::data_local_dir().unwrap().join("openmw"), path);
    }

    #[test]
    #[cfg(windows)]
    fn default_global_config_dir_on_windows_should_be_in_program_files() {
        let path = default_global_config_dir().unwrap();

        assert!(path.ends_with("Program Files\\OpenMW"));
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_config_dir_on_linux_should_use_game_path_if_it_contains_resources_vfs() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        create_dir_all(game_path.join("resources/vfs")).unwrap();

        let path = default_global_config_dir(game_path);

        assert_eq!(game_path, path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_config_dir_on_linux_flatpak_should_use_etc_folder_in_flatpak_files_dir() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=org.openmw.OpenMW",
        )
        .unwrap();

        let path = default_global_config_dir(&game_path);

        assert_eq!(game_path.join("../etc/openmw"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_config_dir_on_linux_should_use_root_etc_folder_if_not_flatpak() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let path = default_global_config_dir(game_path);

        assert_eq!(Path::new("/etc/openmw"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_data_dir_on_linux_should_use_game_path_if_it_contains_resources_vfs() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        create_dir_all(game_path.join("resources/vfs")).unwrap();

        let path = default_global_data_dir(game_path);

        assert_eq!(game_path, path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_data_dir_on_linux_flatpak_should_use_share_folder_in_flatpak_files_dir() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=org.openmw.OpenMW",
        )
        .unwrap();

        let path = default_global_data_dir(&game_path);

        assert_eq!(game_path.join("../share/games/openmw"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn default_global_data_dir_on_linux_should_use_root_usr_share_games_folder_if_it_exists() {
        // Can't create the directory without root permissions, so just check if it exists and
        // adjust what's asserted accordingly.
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let path = default_global_data_dir(game_path);

        if std::fs::exists("/usr/share/games/openmw").unwrap() {
            assert_eq!(Path::new("/usr/share/games/openmw"), path);
        } else {
            assert_eq!(Path::new("/usr/share/openmw"), path);
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn is_flatpak_install_should_be_true_if_a_flatpak_metadata_file_with_the_right_name_field_exists(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=org.openmw.OpenMW",
        )
        .unwrap();

        assert!(is_flatpak_install(&game_path));
    }

    #[test]
    #[cfg(not(windows))]
    fn is_flatpak_install_should_be_false_if_a_flatpak_metadata_file_with_the_wrong_name_field_exists(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=com.example.Wrong",
        )
        .unwrap();

        assert!(!is_flatpak_install(&game_path));
    }

    #[test]
    #[cfg(not(windows))]
    fn is_flatpak_install_should_be_false_if_a_metadata_file_of_the_wrong_format_exists() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(tmp_dir.path().join("metadata"), "org.openmw.OpenMW").unwrap();

        assert!(!is_flatpak_install(&game_path));
    }

    #[test]
    #[cfg(not(windows))]
    fn is_flatpak_install_should_be_false_if_no_metadata_file_exists() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        assert!(!is_flatpak_install(&game_path));
    }

    #[test]
    #[cfg(windows)]
    fn fixed_paths_new_on_windows_should_share_config_and_data_paths() {
        let game_path = Path::new("game");
        let paths = FixedPaths::new(game_path).unwrap();

        let expected_user_path = dirs::document_dir().unwrap().join("My Games\\OpenMW");

        assert_eq!(game_path, paths.local);
        assert_eq!(expected_user_path, paths.user_config);
        assert_eq!(expected_user_path, paths.user_data);
        assert!(paths.global_config.ends_with("Program Files\\OpenMW"));
        assert_eq!(paths.global_config, paths.global_data);
        assert!(paths.flatpak_app.is_none());
    }

    #[test]
    #[cfg(not(windows))]
    fn fixed_paths_on_linux_should_set_flatpak_app_path_to_none_if_is_not_flatpak_install() {
        let tmp_dir = tempdir().unwrap();

        let paths = FixedPaths::new(tmp_dir.path()).unwrap();

        assert_eq!(tmp_dir.path(), paths.local);
        assert_eq!(
            dirs::config_dir().unwrap().join("openmw"),
            paths.user_config
        );
        assert_eq!(
            dirs::data_local_dir().unwrap().join("openmw"),
            paths.user_data
        );
        assert_eq!(Path::new("/etc/openmw"), paths.global_config);
        assert_eq!(Path::new("/usr/share/openmw"), paths.global_data);
        assert!(paths.flatpak_app.is_none());
    }

    #[test]
    #[cfg(not(windows))]
    fn fixed_paths_on_linux_should_set_flatpak_app_path_to_dir_above_the_game_path_if_is_flatpak_install(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("files/bin");

        create_dir_all(&game_path).unwrap();
        std::fs::write(
            tmp_dir.path().join("metadata"),
            "[Application]\nname=org.openmw.OpenMW",
        )
        .unwrap();

        let paths = FixedPaths::new(&game_path).unwrap();

        assert_eq!(game_path, paths.local);
        assert_eq!(
            dirs::home_dir()
                .unwrap()
                .join(".var/app/org.openmw.OpenMW/config/openmw"),
            paths.user_config
        );
        assert_eq!(
            dirs::home_dir()
                .unwrap()
                .join(".var/app/org.openmw.OpenMW/data/openmw"),
            paths.user_data
        );
        assert_eq!(game_path.join("../etc/openmw"), paths.global_config);
        assert_eq!(game_path.join("../share/games/openmw"), paths.global_data);
        assert_eq!(game_path.join(".."), paths.flatpak_app.unwrap());
    }

    #[test]
    fn openmw_config_reduce_into_should_replace_the_first_resources_only_if_it_is_none() {
        let first = OpenMWConfig::default();
        let second = OpenMWConfig {
            resources: Some("a".into()),
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(Some("a".into()), merged.resources);

        let third = OpenMWConfig {
            resources: Some("b".into()),
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(Some("a".into()), merged.resources);
    }

    #[test]
    fn openmw_config_reduce_into_should_replace_the_first_data_local_only_if_it_is_none() {
        let first = OpenMWConfig::default();
        let second = OpenMWConfig {
            data_local: Some("a".into()),
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(Some("a".into()), merged.data_local);

        let third = OpenMWConfig {
            data_local: Some("b".into()),
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(Some("a".into()), merged.data_local);
    }

    #[test]
    fn openmw_config_reduce_into_should_append_replace_values_unless_second_replace_contains_replace(
    ) {
        let first = OpenMWConfig {
            replace: vec!["a".into()],
            ..Default::default()
        };
        let second = OpenMWConfig {
            replace: vec!["b".into()],
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(vec!["a".to_string(), "b".into()], merged.replace);

        let third = OpenMWConfig {
            replace: vec!["c".into(), "replace".into()],
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(vec!["c".to_string(), "replace".into()], merged.replace);
    }

    #[test]
    fn openmw_config_reduce_into_should_append_config_values_unless_second_replace_contains_config()
    {
        let first = OpenMWConfig {
            config: vec!["a".into()],
            ..Default::default()
        };
        let second = OpenMWConfig {
            config: vec!["b".into()],
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(vec![PathBuf::from("a"), "b".into()], merged.config);

        let third = OpenMWConfig {
            replace: vec!["config".into()],
            config: vec!["c".into()],
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(vec![PathBuf::from("c")], merged.config);
    }

    #[test]
    fn openmw_config_reduce_into_should_append_data_values_unless_second_replace_contains_data() {
        let first = OpenMWConfig {
            data: vec!["a".into()],
            ..Default::default()
        };
        let second = OpenMWConfig {
            data: vec!["b".into()],
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(vec![PathBuf::from("a"), "b".into()], merged.data);

        let third = OpenMWConfig {
            replace: vec!["data".into()],
            data: vec!["c".into()],
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(vec![PathBuf::from("c")], merged.data);
    }

    #[test]
    fn openmw_config_reduce_into_should_append_content_values_unless_second_replace_contains_content(
    ) {
        let first = OpenMWConfig {
            content: vec!["a".into()],
            ..Default::default()
        };
        let second = OpenMWConfig {
            content: vec!["b".into()],
            ..Default::default()
        };
        let merged = second.reduce_into(first);

        assert_eq!(vec!["a".to_string(), "b".into()], merged.content);

        let third = OpenMWConfig {
            replace: vec!["content".into()],
            content: vec!["c".into()],
            ..Default::default()
        };
        let merged = third.reduce_into(merged);

        assert_eq!(vec!["c".to_string()], merged.content);
    }

    #[test]
    fn openmw_config_into_additional_data_paths_should_use_data_local_and_data_paths() {
        let config = OpenMWConfig {
            data_local: Some("a".into()),
            data: vec!["b".into(), "c".into()],
            ..Default::default()
        };

        assert_eq!(
            vec![PathBuf::from("a"), "b".into(), "c".into()],
            config.into_additional_data_paths()
        );
    }

    #[test]
    fn parse_path_value_should_strip_enclosing_double_quotes_and_ampersand_escapes() {
        let parsed = parse_path_value("\"Path\\&&&\"&a&&&&\\Data Files\"");

        assert_eq!("Path\\&\"a&&\\Data Files", parsed);
    }

    #[test]
    fn parse_path_value_should_return_an_unquoted_value_as_is() {
        let value = "&\"&\"";
        let parsed = parse_path_value(value);

        assert_eq!(value, parsed);
    }

    #[test]
    fn resolve_path_value_should_return_empty_path_if_value_is_empty() {
        let resolved = resolve_path_value(String::new(), Path::new(""), &fixed_paths());

        assert_eq!(Some(PathBuf::new()), resolved);
    }

    #[test]
    fn resolve_path_value_should_resolve_a_relative_path_to_the_config_directory() {
        let value = "relative/path";
        let config_dir = Path::new("config/directory");
        let resolved = resolve_path_value(value.to_string(), config_dir, &fixed_paths());

        assert_eq!(Some(config_dir.join(value)), resolved);
    }

    #[test]
    #[cfg(not(windows))]
    fn resolve_path_value_should_replace_app_prefix_if_flatpak_app_path_is_defined() {
        let value = "/app/path/to/somewhere".to_string();
        let resolved = resolve_path_value(value, Path::new(""), &fixed_paths());

        assert_eq!(
            Some(fixed_paths().flatpak_app.unwrap().join("path/to/somewhere")),
            resolved
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn resolve_path_value_should_not_replace_app_prefix_if_flatpak_app_path_is_not_defined() {
        let value = "/app/path/to/somewhere";
        let mut fixed_paths = fixed_paths();
        fixed_paths.flatpak_app = None;
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(value.into()), resolved);
    }

    #[test]
    #[cfg(windows)]
    fn resolve_path_value_should_return_absolute_path_unchanged() {
        let value = "C:\\absolute\\path";
        let config_dir = Path::new("config/directory");
        let resolved = resolve_path_value(value.to_string(), config_dir, &fixed_paths());

        assert_eq!(Some(value.into()), resolved);
    }

    #[test]
    fn resolve_path_value_should_not_replace_token_that_appears_after_the_start_of_the_value() {
        let value = "prefix?userconfig?";
        let config_dir = Path::new("config/directory");
        let resolved = resolve_path_value(value.to_string(), config_dir, &fixed_paths());

        assert_eq!(Some(config_dir.join(value)), resolved);
    }

    #[test]
    fn resolve_path_value_should_replace_local_token_prefix() {
        let value = "?local?suffix";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(fixed_paths.local.join("suffix")), resolved);
    }

    #[test]
    fn resolve_path_value_should_replace_userconfig_token_prefix() {
        let value = "?userconfig?suffix";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(fixed_paths.user_config.join("suffix")), resolved);
    }

    #[test]
    fn resolve_path_value_should_replace_userdata_token_prefix() {
        let value = "?userdata?suffix";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(fixed_paths.user_data.join("suffix")), resolved);
    }

    #[test]
    fn resolve_path_value_should_replace_global_token_prefix() {
        let value = "?global?suffix";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(fixed_paths.global_data.join("suffix")), resolved);
    }

    #[test]
    fn resolve_path_value_should_return_none_if_token_prefix_is_unrecognised() {
        let value = "?other?suffix";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert!(resolved.is_none());
    }

    #[test]
    fn resolve_path_value_should_handle_token_with_no_suffix() {
        let value = "?local?";
        let fixed_paths = fixed_paths();
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths);

        assert_eq!(Some(fixed_paths.local), resolved);
    }

    #[test]
    fn resolve_path_value_should_return_a_value_starting_with_a_question_mark_but_not_containing_another_unchanged(
    ) {
        let value = "?local";
        let resolved = resolve_path_value(value.to_string(), Path::new(""), &fixed_paths());

        assert_eq!(Some(value.into()), resolved);
    }

    #[test]
    fn load_config_should_parse_and_resolve_paths() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(
            &ini_path,
            "data=\"Path\\&&&\"&a&&&&\\Data Files\"\ndata=?local?games/path",
        )
        .unwrap();

        let fixed_paths = fixed_paths();
        let config = load_config(tmp_dir.path(), &fixed_paths).unwrap().unwrap();

        let expected_paths: &[PathBuf] = &[
            tmp_dir.path().join("Path\\&\"a&&\\Data Files"),
            fixed_paths.local.join("games/path"),
        ];
        assert_eq!(expected_paths, config.data);
    }

    #[test]
    fn load_config_should_not_error_if_the_given_path_does_not_exist() {
        let config = load_config(Path::new("missing"), &fixed_paths()).unwrap();

        assert!(config.is_none());
    }

    #[test]
    fn load_game_configs_should_use_global_config_dir_as_user_config_dir_if_no_configs_are_found() {
        let fixed_paths = fixed_paths();
        let state = load_game_configs(&fixed_paths).unwrap();

        assert!(state.loaded_configs.is_empty());
        assert_eq!(fixed_paths.global_config, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_use_local_config_if_present() {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");
        let global_path = tmp_dir.path().join("global");

        std::fs::create_dir(&local_path).unwrap();
        std::fs::create_dir(&global_path).unwrap();

        std::fs::write(local_path.join("openmw.cfg"), "resources=./resources").unwrap();
        std::fs::write(global_path.join("openmw.cfg"), "resources=./other").unwrap();

        let fixed_paths = FixedPaths {
            local: local_path.clone(),
            global_config: global_path,
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![OpenMWConfig {
                resources: Some(local_path.join("resources")),
                ..Default::default()
            }],
            state.loaded_configs
        );
        assert_eq!(fixed_paths.local, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_use_global_config_if_local_config_is_not_present() {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");
        let global_path = tmp_dir.path().join("global");

        std::fs::create_dir(&local_path).unwrap();
        std::fs::create_dir(&global_path).unwrap();

        std::fs::write(global_path.join("openmw.cfg"), "resources=./other").unwrap();

        let fixed_paths = FixedPaths {
            local: local_path,
            global_config: global_path.clone(),
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![OpenMWConfig {
                resources: Some(global_path.join("other")),
                ..Default::default()
            }],
            state.loaded_configs
        );
        assert_eq!(fixed_paths.global_config, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_process_config_entries_in_filo_order_within_each_file() {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");
        let other_path_1 = tmp_dir.path().join("other1");
        let other_path_2 = tmp_dir.path().join("other2");
        let other_path_3 = tmp_dir.path().join("other3");

        std::fs::create_dir(&local_path).unwrap();
        std::fs::create_dir(&other_path_1).unwrap();
        std::fs::create_dir(&other_path_2).unwrap();
        std::fs::create_dir(&other_path_3).unwrap();

        std::fs::write(
            local_path.join("openmw.cfg"),
            format!(
                "config=\"{}\"\nconfig=\"{}\"\ncontent=a",
                other_path_1.to_str().unwrap(),
                other_path_2.to_str().unwrap()
            ),
        )
        .unwrap();
        std::fs::write(
            other_path_1.join("openmw.cfg"),
            format!(
                "config=\"{}\"\ncontent=b\ncontent=c",
                other_path_3.to_str().unwrap()
            ),
        )
        .unwrap();
        std::fs::write(other_path_2.join("openmw.cfg"), "content=d").unwrap();
        std::fs::write(other_path_3.join("openmw.cfg"), "content=e").unwrap();

        let fixed_paths = FixedPaths {
            local: local_path.clone(),
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![
                OpenMWConfig {
                    config: vec![other_path_1.clone(), other_path_2.clone()],
                    content: vec!["a".to_string()],
                    ..Default::default()
                },
                OpenMWConfig {
                    content: vec!["d".to_string()],
                    ..Default::default()
                },
                OpenMWConfig {
                    config: vec![other_path_3.clone()],
                    content: vec!["b".to_string(), "c".to_string()],
                    ..Default::default()
                },
                OpenMWConfig {
                    content: vec!["e".to_string()],
                    ..Default::default()
                }
            ],
            state.loaded_configs
        );
        assert_eq!(other_path_3, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_truncate_loaded_configs_when_it_encounters_a_replace_config_entry()
    {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");
        let other_path_1 = tmp_dir.path().join("other1");
        let other_path_2 = tmp_dir.path().join("other2");
        let other_path_3 = tmp_dir.path().join("other3");

        std::fs::create_dir(&local_path).unwrap();
        std::fs::create_dir(&other_path_1).unwrap();
        std::fs::create_dir(&other_path_2).unwrap();
        std::fs::create_dir(&other_path_3).unwrap();

        std::fs::write(
            local_path.join("openmw.cfg"),
            format!(
                "config=\"{}\"\nconfig=\"{}\"\ncontent=a",
                other_path_1.to_str().unwrap(),
                other_path_2.to_str().unwrap()
            ),
        )
        .unwrap();
        std::fs::write(
            other_path_1.join("openmw.cfg"),
            format!(
                "config=\"{}\"\ncontent=b\ncontent=c",
                other_path_3.to_str().unwrap()
            ),
        )
        .unwrap();
        std::fs::write(other_path_2.join("openmw.cfg"), "content=d").unwrap();
        std::fs::write(other_path_3.join("openmw.cfg"), "content=e\nreplace=config").unwrap();

        let fixed_paths = FixedPaths {
            local: local_path.clone(),
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![
                OpenMWConfig {
                    config: vec![other_path_1.clone(), other_path_2.clone()],
                    content: vec!["a".to_string()],
                    ..Default::default()
                },
                OpenMWConfig {
                    replace: vec!["config".to_string()],
                    content: vec!["e".to_string()],
                    ..Default::default()
                }
            ],
            state.loaded_configs
        );
        assert_eq!(other_path_3, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_record_empty_config_if_a_referenced_path_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");
        let other_path = tmp_dir.path().join("other1");

        std::fs::create_dir(&local_path).unwrap();

        std::fs::write(
            local_path.join("openmw.cfg"),
            format!("config=\"{}\"\ncontent=a", other_path.to_str().unwrap()),
        )
        .unwrap();

        let fixed_paths = FixedPaths {
            local: local_path.clone(),
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![
                OpenMWConfig {
                    config: vec![other_path.clone()],
                    content: vec!["a".to_string()],
                    ..Default::default()
                },
                OpenMWConfig::default()
            ],
            state.loaded_configs
        );
        assert_eq!(other_path, state.user_config_dir);
    }

    #[test]
    fn load_game_configs_should_not_get_stuck_in_a_loop() {
        let tmp_dir = tempdir().unwrap();
        let local_path = tmp_dir.path().join("local");

        std::fs::create_dir(&local_path).unwrap();

        std::fs::write(
            local_path.join("openmw.cfg"),
            format!("config=\"{}\"\ncontent=a", local_path.to_str().unwrap()),
        )
        .unwrap();

        let fixed_paths = FixedPaths {
            local: local_path.clone(),
            ..fixed_paths()
        };
        let state = load_game_configs(&fixed_paths).unwrap();

        assert_eq!(
            vec![OpenMWConfig {
                config: vec![local_path.clone()],
                content: vec!["a".to_string()],
                ..Default::default()
            }],
            state.loaded_configs
        );
        assert_eq!(local_path, state.user_config_dir);
    }

    #[test]
    fn load_game_config_with_user_config_dir_should_replace_last_loaded_config() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let other_path_1 = tmp_dir.path().join("other1");
        let other_path_2 = tmp_dir.path().join("other2");

        std::fs::create_dir(&game_path).unwrap();
        std::fs::create_dir(&other_path_1).unwrap();
        std::fs::create_dir(&other_path_2).unwrap();

        std::fs::write(
            game_path.join("openmw.cfg"),
            format!("config=\"{}\"\ncontent=a", other_path_1.to_str().unwrap()),
        )
        .unwrap();
        std::fs::write(other_path_1.join("openmw.cfg"), "content=b").unwrap();
        std::fs::write(other_path_2.join("openmw.cfg"), "content=c").unwrap();

        let config = load_game_config_with_user_config_dir(&game_path, &other_path_2).unwrap();

        assert_eq!(
            OpenMWConfig {
                config: vec![other_path_1.clone()],
                content: vec!["a".to_string(), "c".to_string()],
                ..Default::default()
            },
            config
        );
    }

    #[test]
    fn load_game_config_with_user_config_dir_should_replace_last_loaded_config_even_if_the_given_dir_has_no_config(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let other_path_1 = tmp_dir.path().join("other1");
        let other_path_2 = tmp_dir.path().join("other2");

        std::fs::create_dir(&game_path).unwrap();
        std::fs::create_dir(&other_path_1).unwrap();
        std::fs::create_dir(&other_path_2).unwrap();

        std::fs::write(
            game_path.join("openmw.cfg"),
            format!("config=\"{}\"\ncontent=a", other_path_1.to_str().unwrap()),
        )
        .unwrap();
        std::fs::write(other_path_1.join("openmw.cfg"), "content=b").unwrap();

        let config = load_game_config_with_user_config_dir(&game_path, &other_path_2).unwrap();

        assert_eq!(
            OpenMWConfig {
                config: vec![other_path_1.clone()],
                content: vec!["a".to_string()],
                ..Default::default()
            },
            config
        );
    }

    #[test]
    fn load_non_user_config_should_drop_the_last_loaded_config() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let other_path = tmp_dir.path().join("other");

        std::fs::create_dir(&game_path).unwrap();
        std::fs::create_dir(&other_path).unwrap();

        std::fs::write(
            game_path.join("openmw.cfg"),
            format!("config=\"{}\"\ncontent=a", other_path.to_str().unwrap()),
        )
        .unwrap();
        std::fs::write(other_path.join("openmw.cfg"), "content=b").unwrap();

        let config = load_non_user_config(&game_path).unwrap();

        assert_eq!(
            OpenMWConfig {
                config: vec![other_path.clone()],
                content: vec!["a".to_string()],
                ..Default::default()
            },
            config
        );
    }

    #[test]
    fn write_openmw_cfg_should_write_data_and_content_entries() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(&ini_path, "").unwrap();

        let data_paths = &["C:\\Path\\&\"a&&\\Data Files".into(), "/games/path".into()];
        let active_plugin_names = &["a", "b", "c"];
        write_openmw_cfg(&ini_path, data_paths, active_plugin_names).unwrap();

        let file_content = std::fs::read_to_string(ini_path).unwrap();
        let lines: Vec<_> = file_content.lines().collect();

        assert_eq!(
            vec![
                "data=\"C:\\Path\\&&&\"a&&&&\\Data Files\"",
                "data=\"/games/path\"",
                "content=a",
                "content=b",
                "content=c"
            ],
            lines
        );
    }

    #[test]
    fn write_openmw_cfg_should_preserve_existing_entries_other_than_data_and_content() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(
            &ini_path,
            "key1=value1\ndata=foo\nkey2=value2\nkey2=value3\ncontent=a\ncontent=b\ncontent=c\nkey3=value3")
        .unwrap();

        write_openmw_cfg(&ini_path, &[], &[]).unwrap();

        let file_content = std::fs::read_to_string(ini_path).unwrap();
        let lines: Vec<_> = file_content.lines().collect();

        assert_eq!(
            vec!["key1=value1", "key2=value2", "key2=value3", "key3=value3"],
            lines
        );
    }

    #[test]
    fn write_openmw_cfg_should_not_error_if_the_given_path_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        write_openmw_cfg(&ini_path, &["foo".into()], &["bar"]).unwrap();

        let file_content = std::fs::read_to_string(ini_path).unwrap();
        let lines: Vec<_> = file_content.lines().collect();

        assert_eq!(vec!["data=\"foo\"", "content=bar"], lines);
    }

    #[test]
    fn write_openmw_cfg_should_create_parent_path_if_it_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("a/b/c/d/openmw.cfg");

        assert!(write_openmw_cfg(&ini_path, &["foo".into()], &["bar"]).is_ok());
    }

    #[test]
    fn write_openmw_cfg_strips_comments() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(&ini_path, "#Comment").unwrap();

        write_openmw_cfg(&ini_path, &[], &[]).unwrap();

        let file_content = std::fs::read_to_string(ini_path).unwrap();

        assert_eq!("", file_content);
    }
}
