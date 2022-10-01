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

    pub fn with_local_path(
        game_id: GameId,
        game_path: &Path,
        local_path: &Path,
    ) -> Result<GameSettings, Error> {
        let plugins_file_path = plugins_file_path(game_id, game_path, local_path);
        let load_order_path = load_order_path(game_id, local_path);
        let implicitly_active_plugins = implicitly_active_plugins(game_id, game_path)?;

        Ok(GameSettings {
            id: game_id,
            game_path: game_path.to_path_buf(),
            plugins_file_path,
            load_order_path,
            implicitly_active_plugins,
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
}

// The local path can vary depending on where the game was bought from.
// Skyrim SE and Fallout 4 both have a " MS" suffix added to their local folder
// name when bought from the Microsoft Store, and Skyrim SE has a " GOG" suffix
// added when bought from GOG.
// The logic for checking if a given game install path is for a MS Store game is
// complicated, costly and not fully understood, so don't attempt it here.
// However, the logic for checking if a Skyrim SE install is from GOG is simple
// and relatively quick, so can be done here.
fn appdata_folder_name(game_id: GameId, game_path: &Path) -> Option<&'static str> {
    use crate::enums::GameId::*;
    match game_id {
        Morrowind => None,
        Oblivion => Some("Oblivion"),
        Skyrim => Some("Skyrim"),
        SkyrimSE => Some(skyrim_se_appdata_folder_name(game_path)),
        SkyrimVR => Some("Skyrim VR"),
        Fallout3 => Some("Fallout3"),
        FalloutNV => Some("FalloutNV"),
        Fallout4 => Some("Fallout4"),
        Fallout4VR => Some("Fallout4VR"),
    }
}

fn skyrim_se_appdata_folder_name(game_path: &Path) -> &'static str {
    // Galaxy64.dll is installed by GOG's installer but not by Steam.
    if game_path.join("Galaxy64.dll").exists() {
        "Skyrim Special Edition GOG"
    } else {
        "Skyrim Special Edition"
    }
}

fn load_order_path(game_id: GameId, local_path: &Path) -> Option<PathBuf> {
    match game_id {
        GameId::Skyrim => Some(local_path.join("loadorder.txt")),
        _ => None,
    }
}

fn plugins_file_path(game_id: GameId, game_path: &Path, local_path: &Path) -> PathBuf {
    match game_id {
        GameId::Morrowind => game_path.join("Morrowind.ini"),
        GameId::Oblivion => oblivion_plugins_file_path(game_path, local_path),
        _ => local_path.join("plugins.txt"),
    }
}

fn oblivion_plugins_file_path(game_path: &Path, local_path: &Path) -> PathBuf {
    let ini_path = game_path.join("Oblivion.ini");

    let parent_path = if use_my_games_directory(&ini_path) {
        local_path
    } else {
        game_path
    };

    parent_path.join("plugins.txt")
}

fn use_my_games_directory(ini_path: &Path) -> bool {
    let contents = match std::fs::read(ini_path) {
        // If the ini file isn't present or can't be read, My Games is used by
        // default.
        Err(_) => return true,
        Ok(x) => x,
    };

    // My Games is used if bUseMyGamesDirectory is not present or set to 1.
    !WINDOWS_1252
        .decode_without_bom_handling(&contents)
        .0
        .contains("bUseMyGamesDirectory=0")
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

    Ok(plugin_names)
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::env;
    use std::io::Write;
    use tempfile::tempdir;

    use super::*;

    fn game_with_ccc_plugins(
        game_id: GameId,
        game_path: &Path,
        plugin_names: &[&str],
    ) -> GameSettings {
        let mut file = File::create(ccc_file_path(game_id, &game_path).unwrap()).unwrap();

        for plugin_name in plugin_names {
            writeln!(file, "{}", plugin_name).unwrap();
        }

        GameSettings::with_local_path(game_id, &game_path, &Path::new("local")).unwrap()
    }

    #[test]
    #[cfg(windows)]
    fn new_should_determine_correct_local_path() {
        let settings = GameSettings::new(GameId::Skyrim, Path::new("game")).unwrap();
        let local_app_data = env::var("LOCALAPPDATA").unwrap();
        let local_app_data_path = Path::new(&local_app_data);

        assert_eq!(
            local_app_data_path.join("Skyrim").join("plugins.txt"),
            *settings.active_plugins_file()
        );
        assert_eq!(
            &local_app_data_path.join("Skyrim").join("loadorder.txt"),
            *settings.load_order_file().as_ref().unwrap()
        );
    }

    #[test]
    fn id_should_be_the_id_the_struct_was_created_with() {
        let settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(GameId::Morrowind, settings.id());
    }

    #[test]
    fn load_order_method_should_be_timestamp_for_tes3_tes4_fo3_and_fonv() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_textfile_for_tes5() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        assert_eq!(LoadOrderMethod::Textfile, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_asterisk_for_tes5se_tes5vr_fo4_and_fo4vr() {
        let mut settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::SkyrimVR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Fallout4VR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());
    }

    #[test]
    fn master_file_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Morrowind.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Oblivion.esm", settings.master_file());

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::SkyrimVR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Fallout3.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("FalloutNV.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Fallout4.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Fallout4VR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
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
    fn plugins_folder_name_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data Files", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::SkyrimVR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Fallout4VR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert_eq!("Data", settings.plugins_folder_name());
    }

    #[test]
    fn active_plugins_file_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("game/Morrowind.ini"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &Path::new("game"), &Path::new("local"))
                .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::SkyrimVR,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Fallout4VR,
            &Path::new("game"),
            &Path::new("local"),
        )
        .unwrap();
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );
    }

    #[test]
    fn active_plugins_file_should_be_in_game_path_for_oblivion_if_ini_setting_is_not_1() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(ini_path, "...\nbUseMyGamesDirectory=0\n...").unwrap();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_path, &Path::new("local"))
                .unwrap();
        assert_eq!(
            game_path.join("plugins.txt"),
            *settings.active_plugins_file()
        );
    }

    #[test]
    fn implicitly_active_plugins_should_be_mapped_from_game_id() {
        let mut settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        let mut plugins = vec!["Skyrim.esm", "Update.esm"];
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = GameSettings::with_local_path(
            GameId::SkyrimVR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "HearthFires.esm",
            "Dragonborn.esm",
            "SkyrimVR.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
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

        settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
        assert!(settings.implicitly_active_plugins().is_empty());

        settings = GameSettings::with_local_path(
            GameId::Fallout4VR,
            &PathBuf::default(),
            &PathBuf::default(),
        )
        .unwrap();
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
    fn is_implicitly_active_should_return_true_iff_the_plugin_is_implicitly_active() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        assert!(settings.is_implicitly_active("Update.esm"));
        assert!(!settings.is_implicitly_active("Test.esm"));
    }

    #[test]
    fn is_implicitly_active_should_match_case_insensitively() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default())
                .unwrap();
        assert!(settings.is_implicitly_active("update.esm"));
    }

    #[test]
    fn plugins_folder_should_be_a_child_of_the_game_path() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, Path::new("game"), &PathBuf::default())
                .unwrap();
        assert_eq!(Path::new("game/Data"), settings.plugins_directory());
    }

    #[test]
    fn load_order_file_should_be_in_local_path_for_skyrim_and_none_for_other_games() {
        let mut settings =
            GameSettings::with_local_path(GameId::Skyrim, Path::new("game"), Path::new("local"))
                .unwrap();
        assert_eq!(
            Path::new("local/loadorder.txt"),
            settings.load_order_file().unwrap()
        );

        settings =
            GameSettings::with_local_path(GameId::SkyrimSE, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Morrowind, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Oblivion, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Fallout3, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::FalloutNV, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Fallout4, Path::new("game"), Path::new("local"))
                .unwrap();
        assert!(settings.load_order_file().is_none());
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_path_does_not_exist() {
        assert!(use_my_games_directory(Path::new("does_not_exist")));
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_is_not_present() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "...\n\n...").unwrap();

        assert!(use_my_games_directory(&ini_path));
    }

    #[test]
    fn use_my_games_directory_should_be_false_if_the_ini_setting_value_is_0() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "...\nbUseMyGamesDirectory=0\n...").unwrap();

        assert!(!use_my_games_directory(&ini_path));
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_value_is_not_0() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(&ini_path, "...\nbUseMyGamesDirectory=1\n...").unwrap();

        assert!(use_my_games_directory(&ini_path));
    }
}
