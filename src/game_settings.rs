/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libloadorder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libloadorder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libloadorder. If not, see <http://www.gnu.org/licenses/>.
 */

use std::cmp::Ordering;
use std::fs::{read_dir, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::iter::once;
use std::path::Path;
use std::path::PathBuf;

use crate::enums::{Error, GameId, LoadOrderMethod};
use crate::ini::{test_files, use_my_games_directory};
use crate::is_enderal;
use crate::load_order::{
    AsteriskBasedLoadOrder, OpenMWLoadOrder, TextfileBasedLoadOrder, TimestampBasedLoadOrder,
    WritableLoadOrder,
};
use crate::openmw_config;
use crate::plugin::{has_plugin_extension, Plugin};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GameSettings {
    id: GameId,
    game_path: PathBuf,
    plugins_directory: PathBuf,
    plugins_file_path: PathBuf,
    my_games_path: PathBuf,
    load_order_path: Option<PathBuf>,
    implicitly_active_plugins: Vec<String>,
    early_loading_plugins: Vec<String>,
    additional_plugins_directories: Vec<PathBuf>,
}

const SKYRIM_HARDCODED_PLUGINS: &[&str] = &["Skyrim.esm"];

const SKYRIM_SE_HARDCODED_PLUGINS: &[&str] = &[
    "Skyrim.esm",
    "Update.esm",
    "Dawnguard.esm",
    "HearthFires.esm",
    "Dragonborn.esm",
];

const SKYRIM_VR_HARDCODED_PLUGINS: &[&str] = &[
    "Skyrim.esm",
    "Update.esm",
    "Dawnguard.esm",
    "HearthFires.esm",
    "Dragonborn.esm",
    "SkyrimVR.esm",
];

const FALLOUT4_HARDCODED_PLUGINS: &[&str] = &[
    "Fallout4.esm",
    "DLCRobot.esm",
    "DLCworkshop01.esm",
    "DLCCoast.esm",
    "DLCworkshop02.esm",
    "DLCworkshop03.esm",
    "DLCNukaWorld.esm",
    "DLCUltraHighResolution.esm",
];

const FALLOUT4VR_HARDCODED_PLUGINS: &[&str] = &["Fallout4.esm", "Fallout4_VR.esm"];

pub(crate) const STARFIELD_HARDCODED_PLUGINS: &[&str] = &[
    "Starfield.esm",
    "Constellation.esm",
    "OldMars.esm",
    "ShatteredSpace.esm",
    "SFBGS003.esm",
    "SFBGS004.esm",
    "SFBGS006.esm",
    "SFBGS007.esm",
    "SFBGS008.esm",
];

const OPENMW_HARDCODED_PLUGINS: &[&str] = &["builtin.omwscripts"];

// It's safe to use relative paths like this because the Microsoft Store
// version of Fallout 4 won't launch if a DLC is installed and its install
// path changed (e.g. by renaming a directory), so the DLC plugins must be
// in their default locations.
const MS_FO4_FAR_HARBOR_PATH: &str = "../../Fallout 4- Far Harbor (PC)/Content/Data";
const MS_FO4_NUKA_WORLD_PATH: &str = "../../Fallout 4- Nuka-World (PC)/Content/Data";
const MS_FO4_AUTOMATRON_PATH: &str = "../../Fallout 4- Automatron (PC)/Content/Data";
const MS_FO4_TEXTURE_PACK_PATH: &str = "../../Fallout 4- High Resolution Texture Pack/Content/Data";
const MS_FO4_WASTELAND_PATH: &str = "../../Fallout 4- Wasteland Workshop (PC)/Content/Data";
const MS_FO4_CONTRAPTIONS_PATH: &str = "../../Fallout 4- Contraptions Workshop (PC)/Content/Data";
const MS_FO4_VAULT_TEC_PATH: &str = "../../Fallout 4- Vault-Tec Workshop (PC)/Content/Data";

const PLUGINS_TXT: &str = "Plugins.txt";

impl GameSettings {
    pub fn new(game_id: GameId, game_path: &Path) -> Result<GameSettings, Error> {
        let local_path = local_path(game_id, game_path)?.unwrap_or_default();
        GameSettings::with_local_path(game_id, game_path, &local_path)
    }

    pub fn with_local_path(
        game_id: GameId,
        game_path: &Path,
        local_path: &Path,
    ) -> Result<GameSettings, Error> {
        let my_games_path = my_games_path(game_id, game_path, local_path)?.unwrap_or_default();

        GameSettings::with_local_and_my_games_paths(game_id, game_path, local_path, my_games_path)
    }

    pub(crate) fn with_local_and_my_games_paths(
        game_id: GameId,
        game_path: &Path,
        local_path: &Path,
        my_games_path: PathBuf,
    ) -> Result<GameSettings, Error> {
        let plugins_file_path = plugins_file_path(game_id, game_path, local_path)?;
        let load_order_path = load_order_path(game_id, local_path);
        let plugins_directory = plugins_directory(game_id, game_path, local_path)?;
        let additional_plugins_directories =
            additional_plugins_directories(game_id, game_path, &my_games_path)?;

        let (early_loading_plugins, implicitly_active_plugins) =
            GameSettings::load_implicitly_active_plugins(
                game_id,
                game_path,
                &my_games_path,
                &plugins_directory,
                &additional_plugins_directories,
            )?;

        Ok(GameSettings {
            id: game_id,
            game_path: game_path.to_path_buf(),
            plugins_directory,
            plugins_file_path,
            load_order_path,
            my_games_path,
            implicitly_active_plugins,
            early_loading_plugins,
            additional_plugins_directories,
        })
    }

    pub fn id(&self) -> GameId {
        self.id
    }

    pub fn load_order_method(&self) -> LoadOrderMethod {
        use crate::enums::GameId::*;
        match self.id {
            OpenMW => LoadOrderMethod::OpenMW,
            Morrowind | Oblivion | Fallout3 | FalloutNV => LoadOrderMethod::Timestamp,
            Skyrim => LoadOrderMethod::Textfile,
            SkyrimSE | SkyrimVR | Fallout4 | Fallout4VR | Starfield => LoadOrderMethod::Asterisk,
        }
    }

    pub fn into_load_order(self) -> Box<dyn WritableLoadOrder + Send + Sync + 'static> {
        match self.load_order_method() {
            LoadOrderMethod::Asterisk => Box::new(AsteriskBasedLoadOrder::new(self)),
            LoadOrderMethod::Textfile => Box::new(TextfileBasedLoadOrder::new(self)),
            LoadOrderMethod::Timestamp => Box::new(TimestampBasedLoadOrder::new(self)),
            LoadOrderMethod::OpenMW => Box::new(OpenMWLoadOrder::new(self)),
        }
    }

    #[deprecated = "The master file is not necessarily of any significance: you should probably use early_loading_plugins() instead."]
    pub fn master_file(&self) -> &'static str {
        use crate::enums::GameId::*;
        match self.id {
            Morrowind | OpenMW => "Morrowind.esm",
            Oblivion => "Oblivion.esm",
            Skyrim | SkyrimSE | SkyrimVR => "Skyrim.esm",
            Fallout3 => "Fallout3.esm",
            FalloutNV => "FalloutNV.esm",
            Fallout4 | Fallout4VR => "Fallout4.esm",
            Starfield => "Starfield.esm",
        }
    }

    pub fn implicitly_active_plugins(&self) -> &[String] {
        &self.implicitly_active_plugins
    }

    pub fn is_implicitly_active(&self, plugin: &str) -> bool {
        use unicase::eq;
        self.implicitly_active_plugins()
            .iter()
            .any(|p| eq(p.as_str(), plugin))
    }

    pub fn early_loading_plugins(&self) -> &[String] {
        &self.early_loading_plugins
    }

    pub fn loads_early(&self, plugin: &str) -> bool {
        use unicase::eq;
        self.early_loading_plugins()
            .iter()
            .any(|p| eq(p.as_str(), plugin))
    }

    pub fn plugins_directory(&self) -> PathBuf {
        self.plugins_directory.clone()
    }

    pub fn active_plugins_file(&self) -> &PathBuf {
        &self.plugins_file_path
    }

    pub fn load_order_file(&self) -> Option<&PathBuf> {
        self.load_order_path.as_ref()
    }

    pub fn additional_plugins_directories(&self) -> &[PathBuf] {
        &self.additional_plugins_directories
    }

    pub fn set_additional_plugins_directories(&mut self, paths: Vec<PathBuf>) {
        self.additional_plugins_directories = paths;
    }

    /// Find installed plugins and return them in their "inactive load order",
    /// which is generally the order in which the game launcher would display
    /// them if they were all inactive, ignoring rules like master files
    /// loading before others and about early-loading plugins.
    pub(crate) fn find_plugins(&self) -> Vec<PathBuf> {
        let main_dir_iter = once(&self.plugins_directory);
        let other_directories_iter = self.additional_plugins_directories.iter();

        // For most games, plugins in the additional directories override any of
        // the same names that appear in the main plugins directory, so check
        // for the additional paths first. For OpenMW the main directory is
        // listed first.
        if self.id == GameId::OpenMW {
            find_plugins_in_directories(main_dir_iter.chain(other_directories_iter), self.id)
        } else {
            find_plugins_in_directories(other_directories_iter.chain(main_dir_iter), self.id)
        }
    }

    pub(crate) fn game_path(&self) -> &Path {
        &self.game_path
    }

    pub fn plugin_path(&self, plugin_name: &str) -> PathBuf {
        plugin_path(
            self.id,
            plugin_name,
            &self.plugins_directory,
            &self.additional_plugins_directories,
        )
    }

    pub fn refresh_implicitly_active_plugins(&mut self) -> Result<(), Error> {
        let (early_loading_plugins, implicitly_active_plugins) =
            GameSettings::load_implicitly_active_plugins(
                self.id,
                &self.game_path,
                &self.my_games_path,
                &self.plugins_directory,
                &self.additional_plugins_directories,
            )?;

        self.early_loading_plugins = early_loading_plugins;
        self.implicitly_active_plugins = implicitly_active_plugins;

        Ok(())
    }

    fn load_implicitly_active_plugins(
        game_id: GameId,
        game_path: &Path,
        my_games_path: &Path,
        plugins_directory: &Path,
        additional_plugins_directories: &[PathBuf],
    ) -> Result<(Vec<String>, Vec<String>), Error> {
        let mut test_files = test_files(game_id, game_path, my_games_path)?;

        if matches!(
            game_id,
            GameId::Fallout4 | GameId::Fallout4VR | GameId::Starfield
        ) {
            // Fallout 4 and Starfield ignore plugins.txt and Fallout4.ccc if there are valid
            // plugins listed as test files, so filter out invalid values.
            test_files.retain(|f| {
                let path = plugin_path(
                    game_id,
                    f,
                    plugins_directory,
                    additional_plugins_directories,
                );
                Plugin::with_path(&path, game_id, false).is_ok()
            });
        }

        let early_loading_plugins =
            early_loading_plugins(game_id, game_path, my_games_path, !test_files.is_empty())?;

        let implicitly_active_plugins =
            implicitly_active_plugins(game_id, game_path, &early_loading_plugins, &test_files)?;

        Ok((early_loading_plugins, implicitly_active_plugins))
    }
}

#[cfg(windows)]
fn local_path(game_id: GameId, game_path: &Path) -> Result<Option<PathBuf>, Error> {
    if game_id == GameId::OpenMW {
        return openmw_config::user_config_dir(game_path).map(Some);
    }

    let local_app_data_path = match dirs::data_local_dir() {
        Some(x) => x,
        None => return Err(Error::NoLocalAppData),
    };

    match appdata_folder_name(game_id, game_path) {
        Some(x) => Ok(Some(local_app_data_path.join(x))),
        None => Ok(None),
    }
}

#[cfg(not(windows))]
fn local_path(game_id: GameId, game_path: &Path) -> Result<Option<PathBuf>, Error> {
    if game_id == GameId::OpenMW {
        return openmw_config::user_config_dir(game_path).map(Some);
    } else if appdata_folder_name(game_id, game_path).is_none() {
        // There is no local path, the value doesn't matter.
        Ok(None)
    } else {
        // A local app data path is needed, but there's no way to get it.
        Err(Error::NoLocalAppData)
    }
}

// The local path can vary depending on where the game was bought from.
fn appdata_folder_name(game_id: GameId, game_path: &Path) -> Option<&'static str> {
    use crate::enums::GameId::*;
    match game_id {
        Morrowind | OpenMW => None,
        Oblivion => Some("Oblivion"),
        Skyrim => Some(skyrim_appdata_folder_name(game_path)),
        SkyrimSE => Some(skyrim_se_appdata_folder_name(game_path)),
        SkyrimVR => Some("Skyrim VR"),
        Fallout3 => Some("Fallout3"),
        FalloutNV => Some(falloutnv_appdata_folder_name(game_path)),
        Fallout4 => Some(fallout4_appdata_folder_name(game_path)),
        Fallout4VR => Some("Fallout4VR"),
        Starfield => Some("Starfield"),
    }
}

fn skyrim_appdata_folder_name(game_path: &Path) -> &'static str {
    if is_enderal(game_path) {
        // It's not actually Skyrim, it's Enderal.
        "enderal"
    } else {
        "Skyrim"
    }
}

fn skyrim_se_appdata_folder_name(game_path: &Path) -> &'static str {
    let is_gog_install = game_path.join("Galaxy64.dll").exists();

    if is_enderal(game_path) {
        if is_gog_install {
            "Enderal Special Edition GOG"
        } else {
            "Enderal Special Edition"
        }
    } else if is_gog_install {
        // Galaxy64.dll is only installed by GOG's installer.
        "Skyrim Special Edition GOG"
    } else if game_path.join("EOSSDK-Win64-Shipping.dll").exists() {
        // EOSSDK-Win64-Shipping.dll is only installed by Epic.
        "Skyrim Special Edition EPIC"
    } else if is_microsoft_store_install(GameId::SkyrimSE, game_path) {
        "Skyrim Special Edition MS"
    } else {
        // If neither file is present it's probably the Steam distribution.
        "Skyrim Special Edition"
    }
}

fn falloutnv_appdata_folder_name(game_path: &Path) -> &'static str {
    if game_path.join("EOSSDK-Win32-Shipping.dll").exists() {
        // EOSSDK-Win32-Shipping.dll is only installed by Epic.
        "FalloutNV_Epic"
    } else {
        "FalloutNV"
    }
}

fn fallout4_appdata_folder_name(game_path: &Path) -> &'static str {
    if is_microsoft_store_install(GameId::Fallout4, game_path) {
        "Fallout4 MS"
    } else if game_path.join("EOSSDK-Win64-Shipping.dll").exists() {
        // EOSSDK-Win64-Shipping.dll is only installed by Epic.
        "Fallout4 EPIC"
    } else {
        "Fallout4"
    }
}

fn my_games_path(
    game_id: GameId,
    game_path: &Path,
    local_path: &Path,
) -> Result<Option<PathBuf>, Error> {
    if game_id == GameId::OpenMW {
        // Use the local path as the my games path, so that both refer to the
        // user config path for OpenMW.
        return Ok(Some(local_path.to_path_buf()));
    }

    my_games_folder_name(game_id, game_path)
        .map(|folder| {
            documents_path(local_path)
                .map(|d| d.join("My Games").join(folder))
                .ok_or_else(|| Error::NoDocumentsPath)
        })
        .transpose()
}

fn my_games_folder_name(game_id: GameId, game_path: &Path) -> Option<&'static str> {
    use crate::enums::GameId::*;
    match game_id {
        OpenMW => Some("OpenMW"),
        Skyrim => Some(skyrim_my_games_folder_name(game_path)),
        // For all other games the name is the same as the AppData\Local folder name.
        _ => appdata_folder_name(game_id, game_path),
    }
}

fn skyrim_my_games_folder_name(game_path: &Path) -> &'static str {
    if is_enderal(game_path) {
        "Enderal"
    } else {
        "Skyrim"
    }
}

fn is_microsoft_store_install(game_id: GameId, game_path: &Path) -> bool {
    const APPX_MANIFEST: &str = "appxmanifest.xml";

    match game_id {
        GameId::Morrowind | GameId::Oblivion | GameId::Fallout3 | GameId::FalloutNV => game_path
            .parent()
            .map(|parent| parent.join(APPX_MANIFEST).exists())
            .unwrap_or(false),
        GameId::SkyrimSE | GameId::Fallout4 | GameId::Starfield => {
            game_path.join(APPX_MANIFEST).exists()
        }
        _ => false,
    }
}

#[cfg(windows)]
fn documents_path(_local_path: &Path) -> Option<PathBuf> {
    dirs::document_dir()
}

#[cfg(not(windows))]
fn documents_path(local_path: &Path) -> Option<PathBuf> {
    // Get the documents path relative to the game's local path, which should end in
    // AppData/Local/<Game>.
    local_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(|p| p.join("Documents"))
        .or_else(|| {
            // Fall back to creating a path that navigates up parent directories. This may give a
            // different result if local_path involves symlinks, and requires local_path to exist.
            Some(local_path.join("../../../Documents"))
        })
}

fn plugins_directory(
    game_id: GameId,
    game_path: &Path,
    local_path: &Path,
) -> Result<PathBuf, Error> {
    match game_id {
        GameId::OpenMW => openmw_config::resources_vfs_path(game_path, local_path),
        GameId::Morrowind => Ok(game_path.join("Data Files")),
        _ => Ok(game_path.join("Data")),
    }
}

fn additional_plugins_directories(
    game_id: GameId,
    game_path: &Path,
    my_games_path: &Path,
) -> Result<Vec<PathBuf>, Error> {
    if game_id == GameId::Fallout4 && is_microsoft_store_install(game_id, game_path) {
        Ok(vec![
            game_path.join(MS_FO4_AUTOMATRON_PATH),
            game_path.join(MS_FO4_NUKA_WORLD_PATH),
            game_path.join(MS_FO4_WASTELAND_PATH),
            game_path.join(MS_FO4_TEXTURE_PACK_PATH),
            game_path.join(MS_FO4_VAULT_TEC_PATH),
            game_path.join(MS_FO4_FAR_HARBOR_PATH),
            game_path.join(MS_FO4_CONTRAPTIONS_PATH),
        ])
    } else if game_id == GameId::Starfield {
        Ok(vec![my_games_path.join("Data")])
    } else if game_id == GameId::OpenMW {
        openmw_config::additional_data_paths(game_path, my_games_path)
    } else {
        Ok(Vec::new())
    }
}

fn load_order_path(game_id: GameId, local_path: &Path) -> Option<PathBuf> {
    match game_id {
        GameId::Skyrim => Some(local_path.join("loadorder.txt")),
        _ => None,
    }
}

fn plugins_file_path(
    game_id: GameId,
    game_path: &Path,
    local_path: &Path,
) -> Result<PathBuf, Error> {
    match game_id {
        GameId::OpenMW => Ok(local_path.join("openmw.cfg")),
        GameId::Morrowind => Ok(game_path.join("Morrowind.ini")),
        GameId::Oblivion => oblivion_plugins_file_path(game_path, local_path),
        // Although the launchers for Fallout 3, Fallout NV and Skyrim all create plugins.txt, the
        // games themselves read Plugins.txt.
        _ => Ok(local_path.join(PLUGINS_TXT)),
    }
}

fn oblivion_plugins_file_path(game_path: &Path, local_path: &Path) -> Result<PathBuf, Error> {
    let ini_path = game_path.join("Oblivion.ini");

    let parent_path = if use_my_games_directory(&ini_path)? {
        local_path
    } else {
        game_path
    };

    // Although Oblivion's launcher creates plugins.txt, the game itself reads Plugins.txt.
    Ok(parent_path.join(PLUGINS_TXT))
}

fn ccc_file_paths(game_id: GameId, game_path: &Path, my_games_path: &Path) -> Vec<PathBuf> {
    match game_id {
        GameId::Fallout4 => vec![game_path.join("Fallout4.ccc")],
        GameId::SkyrimSE => vec![game_path.join("Skyrim.ccc")],
        // If the My Games CCC file is present, it overrides the other, even if empty.
        GameId::Starfield => vec![
            my_games_path.join("Starfield.ccc"),
            game_path.join("Starfield.ccc"),
        ],
        _ => vec![],
    }
}

fn hardcoded_plugins(game_id: GameId) -> &'static [&'static str] {
    match game_id {
        GameId::Skyrim => SKYRIM_HARDCODED_PLUGINS,
        GameId::SkyrimSE => SKYRIM_SE_HARDCODED_PLUGINS,
        GameId::SkyrimVR => SKYRIM_VR_HARDCODED_PLUGINS,
        GameId::Fallout4 => FALLOUT4_HARDCODED_PLUGINS,
        GameId::Fallout4VR => FALLOUT4VR_HARDCODED_PLUGINS,
        GameId::Starfield => STARFIELD_HARDCODED_PLUGINS,
        GameId::OpenMW => OPENMW_HARDCODED_PLUGINS,
        _ => &[],
    }
}

fn find_nam_plugins(plugins_path: &Path) -> Result<Vec<String>, Error> {
    // Scan the path for .nam files. Each .nam file can activate a .esm or .esp
    // plugin with the same basename, so return those filenames.
    let mut plugin_names = Vec::new();

    if !plugins_path.exists() {
        return Ok(plugin_names);
    }

    let dir_iter = plugins_path
        .read_dir()
        .map_err(|e| Error::IoError(plugins_path.to_path_buf(), e))?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|f| f.is_file()).unwrap_or(false))
        .filter(|e| {
            e.path()
                .extension()
                .unwrap_or_default()
                .eq_ignore_ascii_case("nam")
        });

    for entry in dir_iter {
        let file_name = entry.file_name();

        let esp = Path::new(&file_name).with_extension("esp");
        if let Some(esp) = esp.to_str() {
            plugin_names.push(esp.to_string());
        }

        let esm = Path::new(&file_name).with_extension("esm");
        if let Some(esm) = esm.to_str() {
            plugin_names.push(esm.to_string());
        }
    }

    Ok(plugin_names)
}

fn early_loading_plugins(
    game_id: GameId,
    game_path: &Path,
    my_games_path: &Path,
    has_test_files: bool,
) -> Result<Vec<String>, Error> {
    let mut plugin_names: Vec<String> = hardcoded_plugins(game_id)
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    if matches!(game_id, GameId::Fallout4 | GameId::Starfield) && has_test_files {
        // If test files are configured for Fallout 4, CCC plugins are not loaded.
        // No need to check for Fallout 4 VR, as it has no CCC plugins file.
        return Ok(plugin_names);
    }

    for file_path in ccc_file_paths(game_id, game_path, my_games_path) {
        if file_path.exists() {
            let reader =
                BufReader::new(File::open(&file_path).map_err(|e| Error::IoError(file_path, e))?);

            let lines = reader
                .lines()
                .filter_map(|line| line.ok().filter(|l| !l.is_empty()));

            plugin_names.extend(lines);
            break;
        }
    }

    if game_id == GameId::OpenMW {
        plugin_names.extend(openmw_config::non_user_active_plugin_names(game_path)?);
    }

    deduplicate(&mut plugin_names);

    Ok(plugin_names)
}

fn implicitly_active_plugins(
    game_id: GameId,
    game_path: &Path,
    early_loading_plugins: &[String],
    test_files: &[String],
) -> Result<Vec<String>, Error> {
    let mut plugin_names = Vec::new();

    plugin_names.extend_from_slice(early_loading_plugins);
    plugin_names.extend_from_slice(test_files);

    if game_id == GameId::FalloutNV {
        // If there is a .nam file with the same basename as a plugin then the plugin is activated
        // and listed as a DLC in the game's title screen menu. This only works in the game's
        // Data path, so ignore additional plugin directories.
        let nam_plugins = find_nam_plugins(&game_path.join("Data"))?;

        plugin_names.extend(nam_plugins);
    } else if game_id == GameId::Skyrim {
        // Update.esm is always active, but loads after all other masters if it is not made to load
        // earlier (e.g. by listing in plugins.txt or by being a master of another master).
        plugin_names.push("Update.esm".to_string());
    } else if game_id == GameId::Starfield {
        // BlueprintShips-Starfield.esm is always active but loads after all other plugins if not
        // made to load earlier.
        plugin_names.push("BlueprintShips-Starfield.esm".to_string());
    }

    deduplicate(&mut plugin_names);

    Ok(plugin_names)
}

/// Remove duplicates, keeping only the first instance of each plugin.
fn deduplicate(plugin_names: &mut Vec<String>) {
    let mut set = std::collections::HashSet::new();
    plugin_names.retain(|e| set.insert(unicase::UniCase::new(e.clone())));
}

fn find_map_path(directory: &Path, plugin_name: &str, game_id: GameId) -> Option<PathBuf> {
    if game_id.allow_plugin_ghosting() {
        // Plugins may be ghosted, so take that into account when checking.
        use crate::ghostable_path::GhostablePath;

        directory.join(plugin_name).resolve_path().ok()
    } else {
        let path = directory.join(plugin_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

fn pick_plugin_path<'a>(
    game_id: GameId,
    plugin_name: &str,
    plugins_directory: &Path,
    mut dir_iter: impl Iterator<Item = &'a PathBuf>,
) -> PathBuf {
    dir_iter
        .find_map(|d| find_map_path(d, plugin_name, game_id))
        .unwrap_or_else(|| plugins_directory.join(plugin_name))
}

fn plugin_path(
    game_id: GameId,
    plugin_name: &str,
    plugins_directory: &Path,
    additional_plugins_directories: &[PathBuf],
) -> PathBuf {
    // There may be multiple directories that the plugin could be installed in, so check each in
    // turn.

    // Starfield (at least as of 1.12.32.0) only loads plugins from its additional directory if
    // they're also present in plugins_directory, so there's no point checking the additional
    // directory if a plugin isn't present in plugins_directory.
    if game_id == GameId::Starfield {
        // Plugins may be ghosted, so take that into account when checking.
        use crate::ghostable_path::GhostablePath;

        let path = plugins_directory.join(plugin_name);
        if path.resolve_path().is_err() {
            return path;
        }
    }

    // In OpenMW, if there are multiple directories containing the same filename, the last directory
    // listed "wins".
    match game_id {
        GameId::OpenMW => pick_plugin_path(
            game_id,
            plugin_name,
            plugins_directory,
            additional_plugins_directories.iter().rev(),
        ),
        _ => pick_plugin_path(
            game_id,
            plugin_name,
            plugins_directory,
            additional_plugins_directories.iter(),
        ),
    }
}

fn sort_plugins_dir_entries(a: &DirEntry, b: &DirEntry) -> Ordering {
    // Sort by file modification timestamps, in ascending order. If two
    // timestamps are equal, sort by filenames in descending order.
    let m_a = a.metadata().and_then(|m| m.modified()).ok();
    let m_b = b.metadata().and_then(|m| m.modified()).ok();

    match m_a.cmp(&m_b) {
        Ordering::Equal => a.file_name().cmp(&b.file_name()).reverse(),
        x => x,
    }
}

fn sort_plugins_dir_entries_starfield(a: &DirEntry, b: &DirEntry) -> Ordering {
    // Sort by file modification timestamps, in ascending order. If two
    // timestamps are equal, sort by filenames in ascending order.
    let m_a = a.metadata().and_then(|m| m.modified()).ok();
    let m_b = b.metadata().and_then(|m| m.modified()).ok();

    match m_a.cmp(&m_b) {
        Ordering::Equal => a.file_name().cmp(&b.file_name()),
        x => x,
    }
}

fn sort_plugins_dir_entries_openmw(a: &DirEntry, b: &DirEntry) -> Ordering {
    // Preserve the directory ordering, but sort case-sensitive
    // lexicographically within directories.
    if a.path().parent() == b.path().parent() {
        a.file_name().cmp(&b.file_name())
    } else {
        Ordering::Equal
    }
}

fn find_plugins_in_directories<'a>(
    directories_iter: impl Iterator<Item = &'a PathBuf>,
    game_id: GameId,
) -> Vec<PathBuf> {
    let mut dir_entries: Vec<_> = directories_iter
        .flat_map(read_dir)
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|f| f.is_file()).unwrap_or(false))
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|f| has_plugin_extension(f, game_id))
                .unwrap_or(false)
        })
        .collect();

    let compare = match game_id {
        GameId::OpenMW => sort_plugins_dir_entries_openmw,
        GameId::Starfield => sort_plugins_dir_entries_starfield,
        _ => sort_plugins_dir_entries,
    };

    dir_entries.sort_by(compare);

    dir_entries.into_iter().map(|e| e.path()).collect()
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::env;
    use std::{
        fs::{create_dir, create_dir_all},
        io::Write,
    };
    use tempfile::tempdir;

    use crate::tests::{copy_to_dir, set_file_timestamps};

    use super::*;

    fn game_with_generic_paths(game_id: GameId) -> GameSettings {
        GameSettings::with_local_and_my_games_paths(
            game_id,
            &PathBuf::from("game"),
            &PathBuf::from("local"),
            PathBuf::from("my games"),
        )
        .unwrap()
    }

    fn game_with_game_path(game_id: GameId, game_path: &Path) -> GameSettings {
        GameSettings::with_local_and_my_games_paths(
            game_id,
            game_path,
            &PathBuf::default(),
            PathBuf::default(),
        )
        .unwrap()
    }

    fn game_with_ccc_plugins(
        game_id: GameId,
        game_path: &Path,
        plugin_names: &[&str],
    ) -> GameSettings {
        let ccc_path = &ccc_file_paths(game_id, game_path, &PathBuf::new())[0];
        create_ccc_file(ccc_path, plugin_names);

        game_with_game_path(game_id, game_path)
    }

    fn create_ccc_file(path: &Path, plugin_names: &[&str]) {
        create_dir_all(path.parent().unwrap()).unwrap();

        let mut file = File::create(path).unwrap();

        for plugin_name in plugin_names {
            writeln!(file, "{}", plugin_name).unwrap();
        }
    }

    #[test]
    #[cfg(windows)]
    fn new_should_determine_correct_local_path_on_windows() {
        let settings = GameSettings::new(GameId::Skyrim, Path::new("game")).unwrap();
        let local_app_data = env::var("LOCALAPPDATA").unwrap();
        let local_app_data_path = Path::new(&local_app_data);

        assert_eq!(
            local_app_data_path.join("Skyrim").join("Plugins.txt"),
            *settings.active_plugins_file()
        );
        assert_eq!(
            &local_app_data_path.join("Skyrim").join("loadorder.txt"),
            *settings.load_order_file().as_ref().unwrap()
        );
    }

    #[test]
    #[cfg(windows)]
    fn new_should_determine_correct_local_path_for_openmw() {
        let tmp_dir = tempdir().unwrap();
        let global_cfg_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(&global_cfg_path, "config=local").unwrap();

        let settings = GameSettings::new(GameId::OpenMW, tmp_dir.path()).unwrap();

        assert_eq!(
            &tmp_dir.path().join("local/openmw.cfg"),
            settings.active_plugins_file()
        );
        assert_eq!(tmp_dir.path().join("local"), settings.my_games_path);
    }

    #[test]
    fn new_should_use_an_empty_local_path_for_morrowind() {
        let settings = GameSettings::new(GameId::Morrowind, Path::new("game")).unwrap();

        assert_eq!(PathBuf::new(), settings.my_games_path);
    }

    #[test]
    #[cfg(not(windows))]
    fn new_should_determine_correct_local_path_for_openmw_on_linux() {
        let config_path = Path::new("/etc/openmw");

        let settings = GameSettings::new(GameId::OpenMW, Path::new("game")).unwrap();

        assert_eq!(
            &config_path.join("openmw.cfg"),
            settings.active_plugins_file()
        );
        assert_eq!(config_path, settings.my_games_path);
    }

    #[test]
    fn id_should_be_the_id_the_struct_was_created_with() {
        let settings = game_with_generic_paths(GameId::Morrowind);
        assert_eq!(GameId::Morrowind, settings.id());
    }

    #[test]
    fn load_order_method_should_be_timestamp_for_tes3_tes4_fo3_and_fonv() {
        let mut settings = game_with_generic_paths(GameId::Morrowind);
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_textfile_for_tes5() {
        let settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!(LoadOrderMethod::Textfile, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_asterisk_for_tes5se_tes5vr_fo4_fo4vr_and_starfield() {
        let mut settings = game_with_generic_paths(GameId::SkyrimSE);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Fallout4);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Starfield);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_openmw_for_openmw() {
        let settings = game_with_generic_paths(GameId::OpenMW);

        assert_eq!(LoadOrderMethod::OpenMW, settings.load_order_method());
    }

    #[test]
    #[allow(deprecated)]
    fn master_file_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::OpenMW);
        assert_eq!("Morrowind.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Morrowind);
        assert_eq!("Morrowind.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert_eq!("Oblivion.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::SkyrimSE);
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert_eq!("Fallout3.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert_eq!("FalloutNV.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Fallout4);
        assert_eq!("Fallout4.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        assert_eq!("Fallout4.esm", settings.master_file());

        settings = game_with_generic_paths(GameId::Starfield);
        assert_eq!("Starfield.esm", settings.master_file());
    }

    #[test]
    fn appdata_folder_name_should_be_mapped_from_game_id() {
        // The game path is unused for most game IDs.
        let game_path = Path::new("");

        assert!(appdata_folder_name(GameId::OpenMW, game_path).is_none());

        assert!(appdata_folder_name(GameId::Morrowind, game_path).is_none());

        let mut folder = appdata_folder_name(GameId::Oblivion, game_path).unwrap();
        assert_eq!("Oblivion", folder);

        folder = appdata_folder_name(GameId::Skyrim, game_path).unwrap();
        assert_eq!("Skyrim", folder);

        folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition", folder);

        folder = appdata_folder_name(GameId::SkyrimVR, game_path).unwrap();
        assert_eq!("Skyrim VR", folder);

        folder = appdata_folder_name(GameId::Fallout3, game_path).unwrap();
        assert_eq!("Fallout3", folder);

        folder = appdata_folder_name(GameId::FalloutNV, game_path).unwrap();
        assert_eq!("FalloutNV", folder);

        folder = appdata_folder_name(GameId::Fallout4, game_path).unwrap();
        assert_eq!("Fallout4", folder);

        folder = appdata_folder_name(GameId::Fallout4VR, game_path).unwrap();
        assert_eq!("Fallout4VR", folder);

        folder = appdata_folder_name(GameId::Starfield, game_path).unwrap();
        assert_eq!("Starfield", folder);
    }

    #[test]
    fn appdata_folder_name_for_skyrim_se_should_have_gog_suffix_if_galaxy_dll_is_in_game_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition", folder);

        let dll_path = game_path.join("Galaxy64.dll");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition GOG", folder);
    }

    #[test]
    fn appdata_folder_name_for_skyrim_se_should_have_epic_suffix_if_eossdk_dll_is_in_game_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition", folder);

        let dll_path = game_path.join("EOSSDK-Win64-Shipping.dll");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition EPIC", folder);
    }

    #[test]
    fn appdata_folder_name_for_skyrim_se_should_have_ms_suffix_if_appxmanifest_xml_is_in_game_path()
    {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition", folder);

        let dll_path = game_path.join("appxmanifest.xml");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition MS", folder);
    }

    #[test]
    fn appdata_folder_name_for_skyrim_se_prefers_gog_suffix_over_epic_suffix() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let dll_path = game_path.join("Galaxy64.dll");
        File::create(&dll_path).unwrap();

        let dll_path = game_path.join("EOSSDK-Win64-Shipping.dll");
        File::create(&dll_path).unwrap();

        let folder = appdata_folder_name(GameId::SkyrimSE, game_path).unwrap();
        assert_eq!("Skyrim Special Edition GOG", folder);
    }

    #[test]
    fn appdata_folder_name_for_fallout_nv_should_have_epic_suffix_if_eossdk_dll_is_in_game_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::FalloutNV, game_path).unwrap();
        assert_eq!("FalloutNV", folder);

        let dll_path = game_path.join("EOSSDK-Win32-Shipping.dll");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::FalloutNV, game_path).unwrap();
        assert_eq!("FalloutNV_Epic", folder);
    }

    #[test]
    fn appdata_folder_name_for_fallout4_should_have_ms_suffix_if_appxmanifest_xml_is_in_game_path()
    {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::Fallout4, game_path).unwrap();
        assert_eq!("Fallout4", folder);

        let dll_path = game_path.join("appxmanifest.xml");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::Fallout4, game_path).unwrap();
        assert_eq!("Fallout4 MS", folder);
    }

    #[test]
    fn appdata_folder_name_for_fallout4_should_have_epic_suffix_if_eossdk_dll_is_in_game_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut folder = appdata_folder_name(GameId::Fallout4, game_path).unwrap();
        assert_eq!("Fallout4", folder);

        let dll_path = game_path.join("EOSSDK-Win64-Shipping.dll");
        File::create(&dll_path).unwrap();

        folder = appdata_folder_name(GameId::Fallout4, game_path).unwrap();
        assert_eq!("Fallout4 EPIC", folder);
    }

    #[test]
    #[cfg(windows)]
    fn my_games_path_should_be_in_documents_path_on_windows() {
        let empty_path = Path::new("");
        let parent_path = dirs::document_dir().unwrap().join("My Games");

        let path = my_games_path(GameId::Morrowind, empty_path, empty_path).unwrap();
        assert!(path.is_none());

        let path = my_games_path(GameId::Oblivion, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Oblivion"), path);

        let path = my_games_path(GameId::Skyrim, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim"), path);

        let path = my_games_path(GameId::SkyrimSE, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim Special Edition"), path);

        let path = my_games_path(GameId::SkyrimVR, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim VR"), path);

        let path = my_games_path(GameId::Fallout3, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout3"), path);

        let path = my_games_path(GameId::FalloutNV, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("FalloutNV"), path);

        let path = my_games_path(GameId::Fallout4, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout4"), path);

        let path = my_games_path(GameId::Fallout4VR, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout4VR"), path);

        let path = my_games_path(GameId::Starfield, empty_path, empty_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Starfield"), path);
    }

    #[test]
    #[cfg(not(windows))]
    fn my_games_path_should_be_relative_to_local_path_on_linux() {
        let empty_path = Path::new("");
        let local_path = Path::new("wineprefix/drive_c/Users/user/AppData/Local/Game");
        let parent_path = Path::new("wineprefix/drive_c/Users/user/Documents/My Games");

        let path = my_games_path(GameId::Morrowind, empty_path, local_path).unwrap();
        assert!(path.is_none());

        let path = my_games_path(GameId::Oblivion, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Oblivion"), path);

        let path = my_games_path(GameId::Skyrim, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim"), path);

        let path = my_games_path(GameId::SkyrimSE, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim Special Edition"), path);

        let path = my_games_path(GameId::SkyrimVR, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Skyrim VR"), path);

        let path = my_games_path(GameId::Fallout3, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout3"), path);

        let path = my_games_path(GameId::FalloutNV, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("FalloutNV"), path);

        let path = my_games_path(GameId::Fallout4, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout4"), path);

        let path = my_games_path(GameId::Fallout4VR, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Fallout4VR"), path);

        let path = my_games_path(GameId::Starfield, empty_path, local_path)
            .unwrap()
            .unwrap();
        assert_eq!(parent_path.join("Starfield"), path);
    }

    #[test]
    #[cfg(windows)]
    fn my_games_path_should_be_local_path_for_openmw() {
        let local_path = Path::new("path/to/local");

        let path = my_games_path(GameId::OpenMW, Path::new(""), local_path)
            .unwrap()
            .unwrap();
        assert_eq!(local_path, path);
    }

    #[test]
    fn plugins_directory_should_be_mapped_from_game_id() {
        let data_path = Path::new("Data");
        let empty_path = Path::new("");
        let closure = |game_id| plugins_directory(game_id, empty_path, empty_path).unwrap();

        assert_eq!(Path::new("resources/vfs"), closure(GameId::OpenMW));
        assert_eq!(Path::new("Data Files"), closure(GameId::Morrowind));
        assert_eq!(data_path, closure(GameId::Oblivion));
        assert_eq!(data_path, closure(GameId::Skyrim));
        assert_eq!(data_path, closure(GameId::SkyrimSE));
        assert_eq!(data_path, closure(GameId::SkyrimVR));
        assert_eq!(data_path, closure(GameId::Fallout3));
        assert_eq!(data_path, closure(GameId::FalloutNV));
        assert_eq!(data_path, closure(GameId::Fallout4));
        assert_eq!(data_path, closure(GameId::Fallout4VR));
        assert_eq!(data_path, closure(GameId::Starfield));
    }

    #[test]
    fn active_plugins_file_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::OpenMW);
        assert_eq!(
            Path::new("local/openmw.cfg"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Morrowind);
        assert_eq!(
            Path::new("game/Morrowind.ini"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Oblivion);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::SkyrimSE);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::SkyrimVR);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Fallout3);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Fallout4);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Fallout4VR);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );

        settings = game_with_generic_paths(GameId::Starfield);
        assert_eq!(
            Path::new("local/Plugins.txt"),
            settings.active_plugins_file()
        );
    }

    #[test]
    fn active_plugins_file_should_be_in_game_path_for_oblivion_if_ini_setting_is_not_1() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(ini_path, "[General]\nbUseMyGamesDirectory=0\n").unwrap();

        let settings = game_with_game_path(GameId::Oblivion, game_path);
        assert_eq!(
            game_path.join("Plugins.txt"),
            *settings.active_plugins_file()
        );
    }

    #[test]
    fn early_loading_plugins_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::Skyrim);
        let mut plugins = vec!["Skyrim.esm"];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::SkyrimSE);
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
        ];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
            "SkyrimVR.esm",
        ];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::Fallout4);
        plugins = vec![
            "Fallout4.esm",
            "DLCRobot.esm",
            "DLCworkshop01.esm",
            "DLCCoast.esm",
            "DLCworkshop02.esm",
            "DLCworkshop03.esm",
            "DLCNukaWorld.esm",
            "DLCUltraHighResolution.esm",
        ];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::OpenMW);
        plugins = vec!["builtin.omwscripts"];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::Morrowind);
        assert!(settings.early_loading_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert!(settings.early_loading_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert!(settings.early_loading_plugins().is_empty());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert!(settings.early_loading_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        plugins = vec!["Fallout4.esm", "Fallout4_VR.esm"];
        assert_eq!(plugins, settings.early_loading_plugins());

        settings = game_with_generic_paths(GameId::Starfield);
        plugins = vec![
            "Starfield.esm",
            "Constellation.esm",
            "OldMars.esm",
            "ShatteredSpace.esm",
            "SFBGS003.esm",
            "SFBGS004.esm",
            "SFBGS006.esm",
            "SFBGS007.esm",
            "SFBGS008.esm",
        ];
        assert_eq!(plugins, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_include_plugins_loaded_from_ccc_file() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
            "ccBGSSSE002-ExoticArrows.esl",
            "ccBGSSSE003-Zombies.esl",
            "ccBGSSSE004-RuinsEdge.esl",
            "ccBGSSSE006-StendarsHammer.esl",
            "ccBGSSSE007-Chrysamere.esl",
            "ccBGSSSE010-PetDwarvenArmoredMudcrab.esl",
            "ccBGSSSE014-SpellPack01.esl",
            "ccBGSSSE019-StaffofSheogorath.esl",
            "ccMTYSSE001-KnightsoftheNine.esl",
            "ccQDRSSE001-SurvivalMode.esl",
        ];
        let mut settings = game_with_ccc_plugins(GameId::SkyrimSE, game_path, &plugins[5..]);
        assert_eq!(plugins, settings.early_loading_plugins());

        plugins = vec![
            "Fallout4.esm",
            "DLCRobot.esm",
            "DLCworkshop01.esm",
            "DLCCoast.esm",
            "DLCworkshop02.esm",
            "DLCworkshop03.esm",
            "DLCNukaWorld.esm",
            "DLCUltraHighResolution.esm",
            "ccBGSFO4001-PipBoy(Black).esl",
            "ccBGSFO4002-PipBoy(Blue).esl",
            "ccBGSFO4003-PipBoy(Camo01).esl",
            "ccBGSFO4004-PipBoy(Camo02).esl",
            "ccBGSFO4006-PipBoy(Chrome).esl",
            "ccBGSFO4012-PipBoy(Red).esl",
            "ccBGSFO4014-PipBoy(White).esl",
            "ccBGSFO4016-Prey.esl",
            "ccBGSFO4017-Mauler.esl",
            "ccBGSFO4018-GaussRiflePrototype.esl",
            "ccBGSFO4019-ChineseStealthArmor.esl",
            "ccBGSFO4020-PowerArmorSkin(Black).esl",
            "ccBGSFO4022-PowerArmorSkin(Camo01).esl",
            "ccBGSFO4023-PowerArmorSkin(Camo02).esl",
            "ccBGSFO4025-PowerArmorSkin(Chrome).esl",
            "ccBGSFO4038-HorseArmor.esl",
            "ccBGSFO4039-TunnelSnakes.esl",
            "ccBGSFO4041-DoomMarineArmor.esl",
            "ccBGSFO4042-BFG.esl",
            "ccBGSFO4043-DoomChainsaw.esl",
            "ccBGSFO4044-HellfirePowerArmor.esl",
            "ccFSVFO4001-ModularMilitaryBackpack.esl",
            "ccFSVFO4002-MidCenturyModern.esl",
            "ccFRSFO4001-HandmadeShotgun.esl",
            "ccEEJFO4001-DecorationPack.esl",
        ];
        settings = game_with_ccc_plugins(GameId::Fallout4, game_path, &plugins[8..]);
        assert_eq!(plugins, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_use_the_starfield_ccc_file_in_game_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let my_games_path = tmp_dir.path().join("my games");

        create_ccc_file(&game_path.join("Starfield.ccc"), &["test.esm"]);

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Starfield,
            &game_path,
            &PathBuf::default(),
            my_games_path,
        )
        .unwrap();

        let expected = &[
            "Starfield.esm",
            "Constellation.esm",
            "OldMars.esm",
            "ShatteredSpace.esm",
            "SFBGS003.esm",
            "SFBGS004.esm",
            "SFBGS006.esm",
            "SFBGS007.esm",
            "SFBGS008.esm",
            "test.esm",
        ];
        assert_eq!(expected, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_use_the_starfield_ccc_file_in_my_games_path() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let my_games_path = tmp_dir.path().join("my games");

        create_ccc_file(&my_games_path.join("Starfield.ccc"), &["test.esm"]);

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Starfield,
            &game_path,
            &PathBuf::default(),
            my_games_path,
        )
        .unwrap();

        let expected = &[
            "Starfield.esm",
            "Constellation.esm",
            "OldMars.esm",
            "ShatteredSpace.esm",
            "SFBGS003.esm",
            "SFBGS004.esm",
            "SFBGS006.esm",
            "SFBGS007.esm",
            "SFBGS008.esm",
            "test.esm",
        ];
        assert_eq!(expected, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_use_the_first_ccc_file_that_exists() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let my_games_path = tmp_dir.path().join("my games");

        create_ccc_file(&game_path.join("Starfield.ccc"), &["test1.esm"]);
        create_ccc_file(&my_games_path.join("Starfield.ccc"), &["test2.esm"]);

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Starfield,
            &game_path,
            &PathBuf::default(),
            my_games_path,
        )
        .unwrap();

        let expected = &[
            "Starfield.esm",
            "Constellation.esm",
            "OldMars.esm",
            "ShatteredSpace.esm",
            "SFBGS003.esm",
            "SFBGS004.esm",
            "SFBGS006.esm",
            "SFBGS007.esm",
            "SFBGS008.esm",
            "test2.esm",
        ];
        assert_eq!(expected, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_not_include_cc_plugins_for_fallout4_if_test_files_are_configured(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        create_ccc_file(
            &game_path.join("Fallout4.ccc"),
            &["ccBGSFO4001-PipBoy(Black).esl"],
        );

        let ini_path = game_path.join("Fallout4.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp\n").unwrap();

        copy_to_dir(
            "Blank.esp",
            &game_path.join("Data"),
            "Blank.esp",
            GameId::Fallout4,
        );

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Fallout4,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        assert_eq!(FALLOUT4_HARDCODED_PLUGINS, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_not_include_cc_plugins_for_starfield_if_test_files_are_configured(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let my_games_path = tmp_dir.path().join("my games");

        create_ccc_file(&my_games_path.join("Starfield.ccc"), &["test.esp"]);

        let ini_path = game_path.join("Starfield.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp\n").unwrap();

        copy_to_dir(
            "Blank.esp",
            &game_path.join("Data"),
            "Blank.esp",
            GameId::Starfield,
        );

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Starfield,
            game_path,
            &PathBuf::default(),
            my_games_path,
        )
        .unwrap();

        assert!(!settings.loads_early("test.esp"));
    }

    #[test]
    fn early_loading_plugins_should_include_plugins_from_global_config_for_openmw() {
        let tmp_dir = tempdir().unwrap();
        let global_cfg_path = tmp_dir.path().join("openmw.cfg");

        std::fs::write(
            &global_cfg_path,
            "config=local\ncontent=test.esm\ncontent=test.esp",
        )
        .unwrap();

        let settings = game_with_game_path(GameId::OpenMW, tmp_dir.path());

        let expected = &["builtin.omwscripts", "test.esm", "test.esp"];

        assert_eq!(expected, settings.early_loading_plugins());
    }

    #[test]
    fn early_loading_plugins_should_ignore_later_duplicate_entries() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let my_games_path = tmp_dir.path().join("my games");

        create_ccc_file(
            &my_games_path.join("Starfield.ccc"),
            &["Starfield.esm", "test.esm"],
        );

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Starfield,
            game_path,
            &PathBuf::default(),
            my_games_path,
        )
        .unwrap();

        let expected = &[
            "Starfield.esm",
            "Constellation.esm",
            "OldMars.esm",
            "ShatteredSpace.esm",
            "SFBGS003.esm",
            "SFBGS004.esm",
            "SFBGS006.esm",
            "SFBGS007.esm",
            "SFBGS008.esm",
            "test.esm",
        ];
        assert_eq!(expected, settings.early_loading_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_include_early_loading_plugins() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let settings = game_with_game_path(GameId::SkyrimSE, game_path);

        assert_eq!(
            settings.early_loading_plugins(),
            settings.implicitly_active_plugins()
        );
    }

    #[test]
    fn implicitly_active_plugins_should_include_test_files() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let ini_path = game_path.join("Skyrim.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=plugin.esp\n").unwrap();

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::SkyrimSE,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        let mut expected_plugins = settings.early_loading_plugins().to_vec();
        expected_plugins.push("plugin.esp".to_string());

        assert_eq!(expected_plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_only_include_valid_test_files_for_fallout4() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let ini_path = game_path.join("Fallout4.ini");
        std::fs::write(
            &ini_path,
            "[General]\nsTestFile1=plugin.esp\nsTestFile2=Blank.esp",
        )
        .unwrap();

        copy_to_dir(
            "Blank.esp",
            &game_path.join("Data"),
            "Blank.esp",
            GameId::Fallout4,
        );

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Fallout4,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        let mut expected_plugins = settings.early_loading_plugins().to_vec();
        expected_plugins.push("Blank.esp".to_string());

        assert_eq!(expected_plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_only_include_valid_test_files_for_fallout4vr() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let ini_path = game_path.join("Fallout4VR.ini");
        std::fs::write(
            &ini_path,
            "[General]\nsTestFile1=plugin.esp\nsTestFile2=Blank.esp",
        )
        .unwrap();

        copy_to_dir(
            "Blank.esp",
            &game_path.join("Data"),
            "Blank.esp",
            GameId::Fallout4VR,
        );

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Fallout4VR,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        let mut expected_plugins = settings.early_loading_plugins().to_vec();
        expected_plugins.push("Blank.esp".to_string());

        assert_eq!(expected_plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_include_plugins_with_nam_files_for_fallout_nv() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let data_path = game_path.join("Data");

        create_dir(&data_path).unwrap();
        File::create(data_path.join("plugin1.nam")).unwrap();
        File::create(data_path.join("plugin2.NAM")).unwrap();

        let settings = game_with_game_path(GameId::FalloutNV, game_path);
        let expected_plugins = vec!["plugin1.esm", "plugin1.esp", "plugin2.esm", "plugin2.esp"];
        let mut plugins = settings.implicitly_active_plugins().to_vec();
        plugins.sort();

        assert_eq!(expected_plugins, plugins);
    }

    #[test]
    fn implicitly_active_plugins_should_include_update_esm_for_skyrim() {
        let settings = game_with_generic_paths(GameId::Skyrim);
        let plugins = settings.implicitly_active_plugins();

        assert!(plugins.contains(&"Update.esm".to_string()));
    }

    #[test]
    fn implicitly_active_plugins_should_include_blueprintships_starfield_esm_for_starfield() {
        let settings = game_with_generic_paths(GameId::Starfield);
        let plugins = settings.implicitly_active_plugins();

        assert!(plugins.contains(&"BlueprintShips-Starfield.esm".to_string()));
    }

    #[test]
    fn implicitly_active_plugins_should_include_plugins_with_nam_files_for_games_other_than_fallout_nv(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let data_path = game_path.join("Data");

        create_dir(&data_path).unwrap();
        File::create(data_path.join("plugin.nam")).unwrap();

        let settings = game_with_game_path(GameId::Fallout3, game_path);
        assert!(settings.implicitly_active_plugins().is_empty());
    }

    #[test]
    fn implicitly_active_plugins_should_not_include_case_insensitive_duplicates() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let ini_path = game_path.join("Fallout4.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=fallout4.esm\n").unwrap();

        let settings = GameSettings::with_local_and_my_games_paths(
            GameId::Fallout4,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        assert_eq!(
            settings.early_loading_plugins(),
            settings.implicitly_active_plugins()
        );
    }

    #[test]
    fn is_implicitly_active_should_return_true_iff_the_plugin_is_implicitly_active() {
        let settings = game_with_generic_paths(GameId::Skyrim);
        assert!(settings.is_implicitly_active("Update.esm"));
        assert!(!settings.is_implicitly_active("Test.esm"));
    }

    #[test]
    fn is_implicitly_active_should_match_case_insensitively() {
        let settings = game_with_generic_paths(GameId::Skyrim);
        assert!(settings.is_implicitly_active("update.esm"));
    }

    #[test]
    fn loads_early_should_return_true_iff_the_plugin_loads_early() {
        let settings = game_with_generic_paths(GameId::SkyrimSE);
        assert!(settings.loads_early("Dawnguard.esm"));
        assert!(!settings.loads_early("Test.esm"));
    }

    #[test]
    fn loads_early_should_match_case_insensitively() {
        let settings = game_with_generic_paths(GameId::SkyrimSE);
        assert!(settings.loads_early("dawnguard.esm"));
    }

    #[test]
    fn plugins_folder_should_be_a_child_of_the_game_path() {
        let settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!(Path::new("game/Data"), settings.plugins_directory());
    }

    #[test]
    fn load_order_file_should_be_in_local_path_for_skyrim_and_none_for_other_games() {
        let mut settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!(
            Path::new("local/loadorder.txt"),
            settings.load_order_file().unwrap()
        );

        settings = game_with_generic_paths(GameId::SkyrimSE);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::OpenMW);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::Morrowind);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert!(settings.load_order_file().is_none());

        settings = game_with_generic_paths(GameId::Fallout4);
        assert!(settings.load_order_file().is_none());
    }

    #[test]
    fn additional_plugins_directories_should_be_empty_if_game_is_not_fallout4_or_starfield() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        File::create(game_path.join("appxmanifest.xml")).unwrap();

        let game_ids = [
            GameId::Morrowind,
            GameId::Oblivion,
            GameId::Skyrim,
            GameId::SkyrimSE,
            GameId::SkyrimVR,
            GameId::Fallout3,
            GameId::FalloutNV,
        ];

        for game_id in game_ids {
            let settings = game_with_game_path(game_id, game_path);

            assert!(settings.additional_plugins_directories().is_empty());
        }
    }

    #[test]
    fn additional_plugins_directories_should_be_empty_if_fallout4_is_not_from_the_microsoft_store()
    {
        let settings = game_with_generic_paths(GameId::Fallout4);

        assert!(settings.additional_plugins_directories().is_empty());
    }

    #[test]
    fn additional_plugins_directories_should_not_be_empty_if_game_is_fallout4_from_the_microsoft_store(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        File::create(game_path.join("appxmanifest.xml")).unwrap();

        let settings = game_with_game_path(GameId::Fallout4, game_path);

        assert_eq!(
            vec![
                game_path.join(MS_FO4_AUTOMATRON_PATH),
                game_path.join(MS_FO4_NUKA_WORLD_PATH),
                game_path.join(MS_FO4_WASTELAND_PATH),
                game_path.join(MS_FO4_TEXTURE_PACK_PATH),
                game_path.join(MS_FO4_VAULT_TEC_PATH),
                game_path.join(MS_FO4_FAR_HARBOR_PATH),
                game_path.join(MS_FO4_CONTRAPTIONS_PATH),
            ],
            settings.additional_plugins_directories()
        );
    }

    #[test]
    fn additional_plugins_directories_should_not_be_empty_if_game_is_starfield() {
        let settings = game_with_generic_paths(GameId::Starfield);

        assert_eq!(
            vec![Path::new("my games").join("Data")],
            settings.additional_plugins_directories()
        );
    }

    #[test]
    fn additional_plugins_directories_should_read_from_openmw_cfgs() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let my_games_path = tmp_dir.path().join("my games");
        let global_cfg_path = game_path.join("openmw.cfg");
        let cfg_path = my_games_path.join("openmw.cfg");

        create_dir_all(global_cfg_path.parent().unwrap()).unwrap();
        std::fs::write(&global_cfg_path, "config=\"../my games\"\ndata=\"foo/bar\"").unwrap();

        create_dir_all(cfg_path.parent().unwrap()).unwrap();
        std::fs::write(
            &cfg_path,
            "data=\"Path\\&&&\"&a&&&&\\Data Files\"\ndata=games/path",
        )
        .unwrap();

        let settings =
            GameSettings::with_local_path(GameId::OpenMW, &game_path, &my_games_path).unwrap();

        let expected: Vec<PathBuf> = vec![
            game_path.join("foo/bar"),
            my_games_path.join("Path\\&\"a&&\\Data Files"),
            my_games_path.join("games/path"),
        ];
        assert_eq!(expected, settings.additional_plugins_directories());
    }

    #[test]
    fn plugin_path_should_append_plugin_name_to_additional_plugin_directory_if_that_path_exists() {
        let tmp_dir = tempdir().unwrap();
        let other_dir = tmp_dir.path().join("other");

        let plugin_name = "external.esp";
        let expected_plugin_path = other_dir.join(plugin_name);

        let mut settings = game_with_generic_paths(GameId::Fallout4);
        settings.additional_plugins_directories = vec![other_dir.clone()];

        copy_to_dir("Blank.esp", &other_dir, plugin_name, GameId::Fallout4);

        let plugin_path = settings.plugin_path(plugin_name);

        assert_eq!(expected_plugin_path, plugin_path);
    }

    #[test]
    fn plugin_path_should_append_plugin_name_to_additional_plugin_directory_if_the_ghosted_path_exists(
    ) {
        let tmp_dir = tempdir().unwrap();
        let other_dir = tmp_dir.path().join("other");

        let plugin_name = "external.esp";
        let ghosted_plugin_name = "external.esp.ghost";
        let expected_plugin_path = other_dir.join(ghosted_plugin_name);

        let mut settings = game_with_generic_paths(GameId::Fallout4);
        settings.additional_plugins_directories = vec![other_dir.clone()];

        copy_to_dir(
            "Blank.esp",
            &other_dir,
            ghosted_plugin_name,
            GameId::Fallout4,
        );

        let plugin_path = settings.plugin_path(plugin_name);

        assert_eq!(expected_plugin_path, plugin_path);
    }

    #[test]
    fn plugin_path_should_not_resolve_ghosted_paths_for_openmw() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let other_dir = tmp_dir.path().join("other");

        let plugin_name = "external.esp";

        let mut settings = game_with_game_path(GameId::OpenMW, &game_path);
        settings.additional_plugins_directories = vec![other_dir.clone()];

        copy_to_dir(
            "Blank.esp",
            &other_dir,
            "external.esp.ghost",
            GameId::OpenMW,
        );

        let plugin_path = settings.plugin_path(plugin_name);

        assert_eq!(
            game_path.join("resources/vfs").join(plugin_name),
            plugin_path
        );
    }

    #[test]
    fn plugin_path_should_return_the_last_directory_that_contains_a_file_for_openmw() {
        let tmp_dir = tempdir().unwrap();
        let other_dir_1 = tmp_dir.path().join("other1");
        let other_dir_2 = tmp_dir.path().join("other2");

        let plugin_name = "Blank.esp";

        let mut settings = game_with_game_path(GameId::OpenMW, tmp_dir.path());
        settings.additional_plugins_directories = vec![other_dir_1.clone(), other_dir_2.clone()];

        copy_to_dir("Blank.esp", &other_dir_1, plugin_name, GameId::OpenMW);
        copy_to_dir("Blank.esp", &other_dir_2, plugin_name, GameId::OpenMW);

        let plugin_path = settings.plugin_path(plugin_name);

        assert_eq!(other_dir_2.join(plugin_name), plugin_path);
    }

    #[test]
    fn plugin_path_should_return_plugins_dir_subpath_if_name_does_not_match_any_external_plugin() {
        let settings = game_with_generic_paths(GameId::Fallout4);

        let plugin_name = "DLCCoast.esm";
        assert_eq!(
            settings.plugins_directory().join(plugin_name),
            settings.plugin_path(plugin_name)
        );
    }

    #[test]
    fn plugin_path_should_only_resolve_additional_starfield_plugin_paths_if_they_exist_or_are_ghosted_in_the_plugins_directory(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("game");
        let data_path = game_path.join("Data");
        let other_dir = tmp_dir.path().join("other");

        let plugin_name_1 = "external1.esp";
        let plugin_name_2 = "external2.esp";
        let plugin_name_3 = "external3.esp";
        let ghosted_plugin_name_3 = "external3.esp.ghost";

        let mut settings = game_with_game_path(GameId::Starfield, &game_path);
        settings.additional_plugins_directories = vec![other_dir.clone()];

        copy_to_dir("Blank.esp", &other_dir, plugin_name_1, GameId::Starfield);
        copy_to_dir("Blank.esp", &other_dir, plugin_name_2, GameId::Starfield);
        copy_to_dir("Blank.esp", &data_path, plugin_name_2, GameId::Starfield);
        copy_to_dir("Blank.esp", &other_dir, plugin_name_3, GameId::Starfield);
        copy_to_dir(
            "Blank.esp",
            &data_path,
            ghosted_plugin_name_3,
            GameId::Starfield,
        );

        let plugin_1_path = settings.plugin_path(plugin_name_1);
        let plugin_2_path = settings.plugin_path(plugin_name_2);
        let plugin_3_path = settings.plugin_path(plugin_name_3);

        assert_eq!(data_path.join(plugin_name_1), plugin_1_path);
        assert_eq!(other_dir.join(plugin_name_2), plugin_2_path);
        assert_eq!(other_dir.join(plugin_name_3), plugin_3_path);
    }

    #[test]
    fn refresh_implicitly_active_plugins_should_update_early_loading_and_implicitly_active_plugins()
    {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let mut settings = GameSettings::with_local_and_my_games_paths(
            GameId::SkyrimSE,
            game_path,
            &PathBuf::default(),
            game_path.to_path_buf(),
        )
        .unwrap();

        let hardcoded_plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
        ];
        assert_eq!(hardcoded_plugins, settings.early_loading_plugins());
        assert_eq!(hardcoded_plugins, settings.implicitly_active_plugins());

        std::fs::write(game_path.join("Skyrim.ccc"), "ccBGSSSE002-ExoticArrows.esl").unwrap();
        std::fs::write(
            game_path.join("Skyrim.ini"),
            "[General]\nsTestFile1=plugin.esp\n",
        )
        .unwrap();

        settings.refresh_implicitly_active_plugins().unwrap();

        let mut expected_plugins = hardcoded_plugins;
        expected_plugins.push("ccBGSSSE002-ExoticArrows.esl");
        assert_eq!(expected_plugins, settings.early_loading_plugins());

        expected_plugins.push("plugin.esp");
        assert_eq!(expected_plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn find_plugins_in_directories_should_sort_files_by_modification_timestamp() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let plugin_names = [
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blàñk.esp",
        ];

        copy_to_dir("Blank.esp", game_path, "Blàñk.esp", GameId::Oblivion);

        for (i, plugin_name) in plugin_names.iter().enumerate() {
            let path = game_path.join(plugin_name);
            if !path.exists() {
                copy_to_dir(plugin_name, game_path, plugin_name, GameId::Oblivion);
            }
            set_file_timestamps(&path, i.try_into().unwrap());
        }

        let result = find_plugins_in_directories(once(&game_path.to_path_buf()), GameId::Oblivion);

        let expected: Vec<_> = plugin_names.iter().map(|n| game_path.join(n)).collect();

        assert_eq!(expected, result);
    }

    #[test]
    fn find_plugins_in_directories_should_sort_files_by_descending_filename_if_timestamps_are_equal(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let plugin_names = [
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blàñk.esp",
        ];

        copy_to_dir("Blank.esp", game_path, "Blàñk.esp", GameId::Oblivion);

        for (i, plugin_name) in plugin_names.iter().enumerate() {
            let path = game_path.join(plugin_name);
            if !path.exists() {
                copy_to_dir(plugin_name, game_path, plugin_name, GameId::Oblivion);
            }
            set_file_timestamps(&path, i.try_into().unwrap());
        }

        let timestamp = 3;
        set_file_timestamps(&game_path.join("Blank - Different.esp"), timestamp);
        set_file_timestamps(&game_path.join("Blank - Master Dependent.esp"), timestamp);

        let result = find_plugins_in_directories(once(&game_path.to_path_buf()), GameId::Oblivion);

        let plugin_paths = vec![
            game_path.join("Blank.esm"),
            game_path.join("Blank.esp"),
            game_path.join("Blank - Master Dependent.esp"),
            game_path.join("Blank - Different.esp"),
            game_path.join("Blàñk.esp"),
        ];

        assert_eq!(plugin_paths, result);
    }

    #[test]
    fn find_plugins_in_directories_should_sort_files_by_ascending_filename_if_timestamps_are_equal_and_game_is_starfield(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();

        let plugin_names = [
            "Blank.full.esm",
            "Blank.small.esm",
            "Blank.medium.esm",
            "Blank.esp",
            "Blank - Override.esp",
        ];

        let timestamp = 1321009991;

        for plugin_name in &plugin_names {
            let path = game_path.join(plugin_name);
            if !path.exists() {
                copy_to_dir(plugin_name, game_path, plugin_name, GameId::Starfield);
            }
            set_file_timestamps(&path, timestamp);
        }

        let result = find_plugins_in_directories(once(&game_path.to_path_buf()), GameId::Starfield);

        let plugin_paths = vec![
            game_path.join("Blank - Override.esp"),
            game_path.join("Blank.esp"),
            game_path.join("Blank.full.esm"),
            game_path.join("Blank.medium.esm"),
            game_path.join("Blank.small.esm"),
        ];

        assert_eq!(plugin_paths, result);
    }
}
