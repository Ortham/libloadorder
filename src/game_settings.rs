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

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use encoding_rs::WINDOWS_1252;

use crate::enums::{Error, GameId, LoadOrderMethod};
use crate::load_order::{
    AsteriskBasedLoadOrder, TextfileBasedLoadOrder, TimestampBasedLoadOrder, WritableLoadOrder,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GameSettings {
    id: GameId,
    game_path: PathBuf,
    plugins_file_path: PathBuf,
    load_order_path: Option<PathBuf>,
    implicitly_active_plugins: Vec<String>,
    additional_plugins_directories: Vec<PathBuf>,
}

const SKYRIM_HARDCODED_PLUGINS: &[&str] = &["Skyrim.esm", "Update.esm"];

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

impl GameSettings {
    #[cfg(windows)]
    pub fn new(game_id: GameId, game_path: &Path) -> Result<GameSettings, Error> {
        let local_app_data_path = app_dirs2::get_data_root(app_dirs2::AppDataType::UserCache)?;
        let local_path = match appdata_folder_name(game_id, game_path) {
            Some(x) => local_app_data_path.join(x),
            None => local_app_data_path,
        };
        GameSettings::with_local_path(game_id, game_path, &local_path)
    }

    #[cfg(not(windows))]
    pub fn new(game_id: GameId, game_path: &Path) -> Result<GameSettings, Error> {
        if appdata_folder_name(game_id, game_path).is_some() {
            Err(Error::NoLocalAppData)
        } else {
            // It doesn't matter what local_path is passed in, as it isn't used.
            GameSettings::with_local_path(game_id, game_path, &PathBuf::new())
        }
    }

    pub fn with_local_path(
        game_id: GameId,
        game_path: &Path,
        local_path: &Path,
    ) -> Result<GameSettings, Error> {
        let plugins_file_path = plugins_file_path(game_id, game_path, local_path)?;
        let load_order_path = load_order_path(game_id, local_path);
        let implicitly_active_plugins = implicitly_active_plugins(game_id, game_path)?;

        Ok(GameSettings {
            id: game_id,
            game_path: game_path.to_path_buf(),
            plugins_file_path,
            load_order_path,
            implicitly_active_plugins,
            additional_plugins_directories: additional_plugins_directories(game_id, game_path),
        })
    }

    pub fn id(&self) -> GameId {
        self.id
    }

    pub fn load_order_method(&self) -> LoadOrderMethod {
        use crate::enums::GameId::*;
        match self.id {
            Morrowind | Oblivion | Fallout3 | FalloutNV => LoadOrderMethod::Timestamp,
            Skyrim => LoadOrderMethod::Textfile,
            SkyrimSE | SkyrimVR | Fallout4 | Fallout4VR => LoadOrderMethod::Asterisk,
        }
    }

    pub fn into_load_order(self) -> Box<dyn WritableLoadOrder> {
        match self.load_order_method() {
            LoadOrderMethod::Asterisk => Box::new(AsteriskBasedLoadOrder::new(self)),
            LoadOrderMethod::Textfile => Box::new(TextfileBasedLoadOrder::new(self)),
            LoadOrderMethod::Timestamp => Box::new(TimestampBasedLoadOrder::new(self)),
        }
    }

    pub fn master_file(&self) -> &'static str {
        use crate::enums::GameId::*;
        match self.id {
            Morrowind => "Morrowind.esm",
            Oblivion => "Oblivion.esm",
            Skyrim | SkyrimSE | SkyrimVR => "Skyrim.esm",
            Fallout3 => "Fallout3.esm",
            FalloutNV => "FalloutNV.esm",
            Fallout4 | Fallout4VR => "Fallout4.esm",
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

    pub fn plugins_directory(&self) -> PathBuf {
        self.game_path.join(self.plugins_folder_name())
    }

    pub fn active_plugins_file(&self) -> &PathBuf {
        &self.plugins_file_path
    }

    pub fn load_order_file(&self) -> Option<&PathBuf> {
        self.load_order_path.as_ref()
    }

    fn plugins_folder_name(&self) -> &'static str {
        match self.id {
            GameId::Morrowind => "Data Files",
            _ => "Data",
        }
    }

    pub fn additional_plugins_directories(&self) -> &[PathBuf] {
        &self.additional_plugins_directories
    }

    pub fn set_additional_plugins_directories(&mut self, paths: Vec<PathBuf>) {
        self.additional_plugins_directories = paths;
    }

    pub fn plugin_path(&self, plugin_name: &str) -> PathBuf {
        // There may be multiple directories that the plugin could be installed in, so check each in turn. Plugins may be ghosted, so take that into account when checking.
        use crate::ghostable_path::GhostablePath;

        self.additional_plugins_directories()
            .iter()
            .find_map(|d| d.join(plugin_name).resolve_path().ok())
            .unwrap_or_else(|| self.plugins_directory().join(plugin_name))
    }
}

// The local path can vary depending on where the game was bought from.
fn appdata_folder_name(game_id: GameId, game_path: &Path) -> Option<&'static str> {
    use crate::enums::GameId::*;
    match game_id {
        Morrowind => None,
        Oblivion => Some("Oblivion"),
        Skyrim => Some(skyrim_appdata_folder_name(game_path)),
        SkyrimSE => Some(skyrim_se_appdata_folder_name(game_path)),
        SkyrimVR => Some("Skyrim VR"),
        Fallout3 => Some("Fallout3"),
        FalloutNV => Some(falloutnv_appdata_folder_name(game_path)),
        Fallout4 => Some(fallout4_appdata_folder_name(game_path)),
        Fallout4VR => Some("Fallout4VR"),
    }
}

fn skyrim_appdata_folder_name(game_path: &Path) -> &'static str {
    if game_path.join("Enderal Launcher.exe").exists() {
        // It's not actually Skyrim, it's Enderal.
        "enderal"
    } else {
        "Skyrim"
    }
}

fn skyrim_se_appdata_folder_name(game_path: &Path) -> &'static str {
    let is_gog_install = game_path.join("Galaxy64.dll").exists();

    if game_path.join("Enderal Launcher.exe").exists() {
        // It's not actually Skyrim SE, it's Enderal SE.
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
    } else {
        "Fallout4"
    }
}

fn is_microsoft_store_install(game_id: GameId, game_path: &Path) -> bool {
    match game_id {
        GameId::Morrowind | GameId::Oblivion | GameId::Fallout3 | GameId::FalloutNV => game_path
            .parent()
            .map(|parent| parent.join("appxmanifest.xml").exists())
            .unwrap_or(false),
        GameId::SkyrimSE | GameId::Fallout4 => game_path.join("appxmanifest.xml").exists(),
        _ => false,
    }
}

fn additional_plugins_directories(game_id: GameId, game_path: &Path) -> Vec<PathBuf> {
    if game_id == GameId::Fallout4 && is_microsoft_store_install(game_id, game_path) {
        vec![
            game_path.join(MS_FO4_AUTOMATRON_PATH),
            game_path.join(MS_FO4_NUKA_WORLD_PATH),
            game_path.join(MS_FO4_WASTELAND_PATH),
            game_path.join(MS_FO4_TEXTURE_PACK_PATH),
            game_path.join(MS_FO4_VAULT_TEC_PATH),
            game_path.join(MS_FO4_FAR_HARBOR_PATH),
            game_path.join(MS_FO4_CONTRAPTIONS_PATH),
        ]
    } else {
        Vec::new()
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
        GameId::Morrowind => Ok(game_path.join("Morrowind.ini")),
        GameId::Oblivion => oblivion_plugins_file_path(game_path, local_path),
        // Although the launchers for Fallout 3, Fallout NV and Skyrim all create plugins.txt, the games themselves read Plugins.txt.
        _ => Ok(local_path.join("Plugins.txt")),
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
    Ok(parent_path.join("Plugins.txt"))
}

fn read_ini(ini_path: &Path) -> Result<ini::Ini, Error> {
    // Read ini as Windows-1252 bytes and then convert to UTF-8 before parsing,
    // as the ini crate expects the content to be valid UTF-8.
    let contents = std::fs::read(ini_path)?;

    // My Games is used if bUseMyGamesDirectory is not present or set to 1.
    let contents = WINDOWS_1252.decode_without_bom_handling(&contents).0;

    ini::Ini::load_from_str(&contents).map_err(Error::from)
}

fn use_my_games_directory(ini_path: &Path) -> Result<bool, Error> {
    if ini_path.exists() {
        // My Games is used if bUseMyGamesDirectory is not present or set to 1.
        read_ini(ini_path)
            .map(|ini| ini.get_from(Some("General"), "bUseMyGamesDirectory") != Some("0"))
    } else {
        Ok(true)
    }
}

fn ccc_file_path(game_id: GameId, game_path: &Path) -> Option<PathBuf> {
    match game_id {
        GameId::Fallout4 => Some(game_path.join("Fallout4.ccc")),
        GameId::SkyrimSE => Some(game_path.join("Skyrim.ccc")),
        _ => None,
    }
}

fn hardcoded_plugins(game_id: GameId) -> &'static [&'static str] {
    match game_id {
        GameId::Skyrim => SKYRIM_HARDCODED_PLUGINS,
        GameId::SkyrimSE => SKYRIM_SE_HARDCODED_PLUGINS,
        GameId::SkyrimVR => SKYRIM_VR_HARDCODED_PLUGINS,
        GameId::Fallout4 => FALLOUT4_HARDCODED_PLUGINS,
        GameId::Fallout4VR => FALLOUT4VR_HARDCODED_PLUGINS,
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
        .read_dir()?
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

fn implicitly_active_plugins(game_id: GameId, game_path: &Path) -> Result<Vec<String>, Error> {
    let mut plugin_names: Vec<String> = hardcoded_plugins(game_id)
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    if let Some(file_path) = ccc_file_path(game_id, game_path) {
        if file_path.exists() {
            let reader = BufReader::new(File::open(file_path)?);

            let lines = reader
                .lines()
                .filter_map(|line| line.ok().filter(|l| !l.is_empty()));

            plugin_names.extend(lines);
        }
    }

    if game_id == GameId::FalloutNV {
        // If there is a .nam file with the same basename as a plugin then the plugin is activated
        // and listed as a DLC in the game's title screen menu. This only works in the game's
        // Data path, so ignore additional plugin directories.
        let nam_plugins = find_nam_plugins(&game_path.join("Data"))?;

        plugin_names.extend(nam_plugins);
    }

    Ok(plugin_names)
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::env;
    use std::{fs::create_dir, io::Write};
    use tempfile::tempdir;

    use crate::tests::copy_to_dir;

    use super::*;

    fn game_with_generic_paths(game_id: GameId) -> GameSettings {
        GameSettings::with_local_path(game_id, &PathBuf::from("game"), &PathBuf::from("local"))
            .unwrap()
    }

    fn game_with_game_path(game_id: GameId, game_path: &Path) -> GameSettings {
        GameSettings::with_local_path(game_id, game_path, &PathBuf::default()).unwrap()
    }

    fn game_with_ccc_plugins(
        game_id: GameId,
        game_path: &Path,
        plugin_names: &[&str],
    ) -> GameSettings {
        let mut file = File::create(ccc_file_path(game_id, &game_path).unwrap()).unwrap();

        for plugin_name in plugin_names {
            writeln!(file, "{}", plugin_name).unwrap();
        }

        game_with_game_path(game_id, game_path)
    }

    #[test]
    #[cfg(windows)]
    fn new_should_determine_correct_local_path() {
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
    fn load_order_method_should_be_asterisk_for_tes5se_tes5vr_fo4_and_fo4vr() {
        let mut settings = game_with_generic_paths(GameId::SkyrimSE);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Fallout4);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());
    }

    #[test]
    fn master_file_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::Morrowind);
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
    }

    #[test]
    fn appdata_folder_name_should_be_mapped_from_game_id() {
        // The game path is unused for most game IDs.
        let game_path = Path::new("");

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
    fn plugins_folder_name_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::Morrowind);
        assert_eq!("Data Files", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::Skyrim);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::SkyrimSE);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::Fallout4);
        assert_eq!("Data", settings.plugins_folder_name());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        assert_eq!("Data", settings.plugins_folder_name());
    }

    #[test]
    fn active_plugins_file_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::Morrowind);
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
    }

    #[test]
    fn active_plugins_file_should_be_in_game_path_for_oblivion_if_ini_setting_is_not_1() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(ini_path, "[General]\nbUseMyGamesDirectory=0\n").unwrap();

        let settings = game_with_game_path(GameId::Oblivion, &game_path);
        assert_eq!(
            game_path.join("Plugins.txt"),
            *settings.active_plugins_file()
        );
    }

    #[test]
    fn implicitly_active_plugins_should_be_mapped_from_game_id() {
        let mut settings = game_with_generic_paths(GameId::Skyrim);
        let mut plugins = vec!["Skyrim.esm", "Update.esm"];
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = game_with_generic_paths(GameId::SkyrimSE);
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = game_with_generic_paths(GameId::SkyrimVR);
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
            "SkyrimVR.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins());

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
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = game_with_generic_paths(GameId::Morrowind);
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Oblivion);
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Fallout3);
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = game_with_generic_paths(GameId::FalloutNV);
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = game_with_generic_paths(GameId::Fallout4VR);
        plugins = vec!["Fallout4.esm", "Fallout4_VR.esm"];
        assert_eq!(plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_include_plugins_loaded_from_ccc_file() {
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
        assert_eq!(plugins, settings.implicitly_active_plugins());

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
        assert_eq!(plugins, settings.implicitly_active_plugins());
    }

    #[test]
    fn implicitly_active_plugins_should_include_plugins_with_nam_files_for_fallout_nv() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let data_path = game_path.join("Data");

        create_dir(&data_path).unwrap();
        File::create(data_path.join("plugin1.nam")).unwrap();
        File::create(data_path.join("plugin2.NAM")).unwrap();

        let settings = game_with_game_path(GameId::FalloutNV, &game_path);
        let expected_plugins = vec!["plugin1.esm", "plugin1.esp", "plugin2.esm", "plugin2.esp"];
        let mut plugins = settings.implicitly_active_plugins().to_vec();
        plugins.sort();

        assert_eq!(expected_plugins, plugins);
    }

    #[test]
    fn implicitly_active_plugins_should_include_plugins_with_nam_files_for_games_other_than_fallout_nv(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let data_path = game_path.join("Data");

        create_dir(&data_path).unwrap();
        File::create(data_path.join("plugin.nam")).unwrap();

        let settings = game_with_game_path(GameId::Fallout3, &game_path);
        assert!(settings.implicitly_active_plugins().is_empty());
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
    fn additional_plugins_directories_should_be_empty_if_game_is_not_fallout4() {
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
    fn plugin_path_should_append_plugin_name_to_additional_plugin_directory_if_that_path_exists() {
        let tmp_dir = tempdir().unwrap();
        let other_dir = tmp_dir.path().join("other");

        let plugin_name = "external.esp";
        let expected_plugin_path = other_dir.join(plugin_name);

        let mut settings = game_with_generic_paths(GameId::Fallout4);
        settings.additional_plugins_directories = vec![other_dir.clone()];

        copy_to_dir("Blank.esp", &other_dir, plugin_name, &settings);

        let plugin_path = settings.plugin_path(plugin_name);

        assert_eq!(expected_plugin_path, plugin_path);
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
    fn read_ini_should_read_empty_values_and_case_insensitive_keys() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(
            &ini_path,
            "[General]\nsTestFile1=\nSTestFile2=a\nsTestFile3=b",
        )
        .unwrap();

        let ini = read_ini(&ini_path).unwrap();

        assert_eq!(Some(""), ini.get_from(Some("General"), "sTestFile1"));
        assert_eq!(Some("a"), ini.get_from(Some("General"), "sTestFile2"));
        assert_eq!(Some("b"), ini.get_from(Some("General"), "sTestFile3"));
        assert_eq!(None, ini.get_from(Some("General"), "sTestFile4"));
    }

    #[test]
    fn read_ini_should_read_as_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(&ini_path, b"[General]\nsTestFile1=\xC0.esp").unwrap();

        let ini = read_ini(&ini_path).unwrap();

        assert_eq!(Some("À.esp"), ini.get_from(Some("General"), "sTestFile1"));
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_path_does_not_exist() {
        assert!(use_my_games_directory(Path::new("does_not_exist")).unwrap());
    }

    #[test]
    fn use_my_games_directory_should_error_if_the_ini_is_invalid() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "[General\nbUseMyGamesDirectory=0").unwrap();

        assert!(use_my_games_directory(&ini_path).is_err());
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_is_not_present() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "[General]\nSStartingCell=").unwrap();

        assert!(use_my_games_directory(&ini_path).unwrap());
    }

    #[test]
    fn use_my_games_directory_should_be_false_if_the_ini_setting_value_is_0() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "[General]\nbUseMyGamesDirectory=0\n").unwrap();

        assert!(!use_my_games_directory(&ini_path).unwrap());
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_value_is_0_but_in_wrong_section() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "[Display]\nbUseMyGamesDirectory=0\n").unwrap();

        assert!(use_my_games_directory(&ini_path).unwrap());
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_value_is_not_0() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "[General]\nbUseMyGamesDirectory=1\n").unwrap();

        assert!(use_my_games_directory(&ini_path).unwrap());
    }
}
