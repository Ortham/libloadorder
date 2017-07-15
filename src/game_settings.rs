use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[cfg(windows)]
use app_dirs;

use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_1252;

use enums::{GameId, LoadOrderMethod};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    NoLocalAppData,
}

#[cfg(windows)]
impl From<app_dirs::AppDirsError> for Error {
    fn from(error: app_dirs::AppDirsError) -> Self {
        match error {
            app_dirs::AppDirsError::Io(x) => Error::IO(x),
            _ => Error::NoLocalAppData,
        }
    }
}

#[derive(Debug)]
pub struct GameSettings {
    id: GameId,
    game_path: PathBuf,
    plugins_file_path: PathBuf,
    load_order_path: Option<PathBuf>,
}

impl GameSettings {
    #[cfg(windows)]
    pub fn new(game_id: GameId, game_path: &Path) -> Result<GameSettings, Error> {
        let local_app_data_path = app_dirs::get_data_root(app_dirs::AppDataType::UserCache)?;
        let local_path = match appdata_folder_name(&game_id) {
            Some(x) => local_app_data_path.join(x),
            None => local_app_data_path,
        };
        Ok(GameSettings::with_local_path(
            game_id,
            game_path,
            &local_path,
        ))
    }

    pub fn with_local_path(game_id: GameId, game_path: &Path, local_path: &Path) -> GameSettings {
        let plugins_file_path = plugins_file_path(&game_id, &game_path, &local_path);
        let load_order_path = load_order_path(&game_id, &local_path);

        GameSettings {
            id: game_id,
            game_path: game_path.to_path_buf(),
            plugins_file_path,
            load_order_path,
        }
    }

    pub fn id(&self) -> &GameId {
        &self.id
    }

    pub fn load_order_method(&self) -> LoadOrderMethod {
        use enums::GameId::*;
        match self.id {
            Morrowind | Oblivion | Fallout3 | FalloutNV => LoadOrderMethod::Timestamp,
            Skyrim => LoadOrderMethod::Textfile,
            SkyrimSE | Fallout4 => LoadOrderMethod::Asterisk,
        }
    }

    pub fn master_file(&self) -> &'static str {
        use enums::GameId::*;
        match self.id {
            Morrowind => "Morrowind.esm",
            Oblivion => "Oblivion.esm",
            Skyrim | SkyrimSE => "Skyrim.esm",
            Fallout3 => "Fallout3.esm",
            FalloutNV => "FalloutNV.esm",
            Fallout4 => "Fallout4.esm",
        }
    }

    pub fn implicitly_active_plugins(&self) -> Option<Vec<&str>> {
        match self.id {
            GameId::Skyrim => Some(vec![self.master_file(), "Update.esm"]),
            GameId::SkyrimSE => Some(vec![
                self.master_file(),
                "Update.esm",
                "Dawnguard.esm",
                "Hearthfires.esm",
                "Dragonborn.esm",
            ]),
            GameId::Fallout4 => Some(vec![
                self.master_file(),
                "DLCRobot.esm",
                "DLCworkshop01.esm",
                "DLCCoast.esm",
                "DLCworkshop02.esm",
                "DLCworkshop03.esm",
                "DLCNukaWorld.esm",
            ]),
            _ => None,
        }
    }

    pub fn is_implicitly_active(&self, plugin: &str) -> bool {
        match self.implicitly_active_plugins() {
            Some(x) => x.contains(&plugin),
            None => false,
        }
    }

    pub fn plugins_folder(&self) -> PathBuf {
        self.game_path.join(self.plugins_folder_name())
    }

    pub fn active_plugins_file(&self) -> &PathBuf {
        &self.plugins_file_path
    }

    pub fn load_order_file(&self) -> &Option<PathBuf> {
        &self.load_order_path
    }

    fn plugins_folder_name(&self) -> &'static str {
        match self.id {
            GameId::Morrowind => "Data Files",
            _ => "Data",
        }
    }
}

fn appdata_folder_name(game_id: &GameId) -> Option<&'static str> {
    use enums::GameId::*;
    match *game_id {
        Morrowind => None,
        Oblivion => Some("Oblivion"),
        Skyrim => Some("Skyrim"),
        SkyrimSE => Some("Skyrim Special Edition"),
        Fallout3 => Some("Fallout3"),
        FalloutNV => Some("FalloutNV"),
        Fallout4 => Some("Fallout4"),
    }
}

fn load_order_path(game_id: &GameId, local_path: &Path) -> Option<PathBuf> {
    match *game_id {
        GameId::Skyrim => Some(local_path.join("loadorder.txt")),
        _ => None,
    }
}

fn plugins_file_path(game_id: &GameId, game_path: &Path, local_path: &Path) -> PathBuf {
    let ini_path = game_path.join("Oblivion.ini");
    match *game_id {
        GameId::Oblivion if ini_path.exists() => {
            if use_my_games_directory(&ini_path) {
                local_path
            } else {
                game_path
            }.join("plugins.txt")
        }
        GameId::Morrowind => game_path.join("Morrowind.ini"),
        _ => local_path.join("plugins.txt"),
    }
}

fn use_my_games_directory(ini_path: &Path) -> bool {
    let file = match File::open(ini_path) {
        Err(_) => return false,
        Ok(x) => x,
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();

    if let Err(_) = buf_reader.read_to_end(&mut contents) {
        return false;
    }

    // Decoding should never fail, since we're just replacing
    // invalid characters and Windows-1252 is a single-byte
    // encoding, but treat it as a 'false' result anyway.
    match WINDOWS_1252.decode(&contents, DecoderTrap::Replace).map(
        |s| {
            s.find("bUseMyGamesDirectory=1")
        },
    ) {
        Err(_) => false,
        Ok(None) => false,
        Ok(Some(_)) => true,
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::env;
    use std::io::Write;
    use self::tempdir::TempDir;

    use super::*;

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
            settings.load_order_file().as_ref().unwrap()
        );
    }

    #[test]
    fn id_should_be_the_id_the_struct_was_created_with() {
        let settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(&GameId::Morrowind, settings.id());
    }

    #[test]
    fn load_order_method_should_be_timestamp_for_tes3_tes4_fo3_and_fonv() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Timestamp, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_textfile_for_tes5() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default());
        assert_eq!(LoadOrderMethod::Textfile, settings.load_order_method());
    }

    #[test]
    fn load_order_method_should_be_asterisk_for_tes5se_and_fo4() {
        let mut settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!(LoadOrderMethod::Asterisk, settings.load_order_method());
    }

    #[test]
    fn master_file_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Morrowind.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Oblivion.esm", settings.master_file());

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default());
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Skyrim.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Fallout3.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("FalloutNV.esm", settings.master_file());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Fallout4.esm", settings.master_file());
    }

    #[test]
    fn appdata_folder_name_should_be_mapped_from_game_id() {
        assert!(appdata_folder_name(&GameId::Morrowind).is_none());

        let mut folder = appdata_folder_name(&GameId::Oblivion).unwrap();
        assert_eq!("Oblivion", folder);

        folder = appdata_folder_name(&GameId::Skyrim).unwrap();
        assert_eq!("Skyrim", folder);

        folder = appdata_folder_name(&GameId::SkyrimSE).unwrap();
        assert_eq!("Skyrim Special Edition", folder);

        folder = appdata_folder_name(&GameId::Fallout3).unwrap();
        assert_eq!("Fallout3", folder);

        folder = appdata_folder_name(&GameId::FalloutNV).unwrap();
        assert_eq!("FalloutNV", folder);

        folder = appdata_folder_name(&GameId::Fallout4).unwrap();
        assert_eq!("Fallout4", folder);
    }

    #[test]
    fn plugins_folder_name_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data Files", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data", settings.plugins_folder_name());

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default());
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data", settings.plugins_folder_name());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert_eq!("Data", settings.plugins_folder_name());
    }

    #[test]
    fn active_plugins_file_should_be_mapped_from_game_id() {
        let mut settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("game/Morrowind.ini"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings =
            GameSettings::with_local_path(GameId::Skyrim, &Path::new("game"), &Path::new("local"));
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &Path::new("game"),
            &Path::new("local"),
        );
        assert_eq!(
            Path::new("local/plugins.txt"),
            settings.active_plugins_file()
        );
    }

    #[test]
    fn active_plugins_file_should_be_in_game_path_for_oblivion_if_ini_setting_is_not_1() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");
        let mut file = File::create(&ini_path).unwrap();
        file.write_all("...\nbUseMyGamesDirectory=0\n...".as_bytes())
            .unwrap();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_path, &Path::new("local"));
        assert_eq!(
            game_path.join("plugins.txt"),
            *settings.active_plugins_file()
        );
    }

    #[test]
    fn implicitly_active_plugins_should_be_mapped_from_game_id() {
        let mut settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default());
        let mut plugins = vec!["Skyrim.esm", "Update.esm"];
        assert_eq!(plugins, settings.implicitly_active_plugins().unwrap());

        settings = GameSettings::with_local_path(
            GameId::SkyrimSE,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        plugins = vec![
            "Skyrim.esm",
            "Update.esm",
            "Dawnguard.esm",
            "Hearthfires.esm",
            "Dragonborn.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins().unwrap());

        settings = GameSettings::with_local_path(
            GameId::Fallout4,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        plugins = vec![
            "Fallout4.esm",
            "DLCRobot.esm",
            "DLCworkshop01.esm",
            "DLCCoast.esm",
            "DLCworkshop02.esm",
            "DLCworkshop03.esm",
            "DLCNukaWorld.esm",
        ];
        assert_eq!(plugins, settings.implicitly_active_plugins().unwrap());

        settings = GameSettings::with_local_path(
            GameId::Morrowind,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert!(settings.implicitly_active_plugins().is_none());

        settings = GameSettings::with_local_path(
            GameId::Oblivion,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert!(settings.implicitly_active_plugins().is_none());

        settings = GameSettings::with_local_path(
            GameId::Fallout3,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert!(settings.implicitly_active_plugins().is_none());

        settings = GameSettings::with_local_path(
            GameId::FalloutNV,
            &PathBuf::default(),
            &PathBuf::default(),
        );
        assert!(settings.implicitly_active_plugins().is_none());
    }

    #[test]
    fn is_implicitly_active_should_return_true_iff_the_plugin_is_implicitly_active() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, &PathBuf::default(), &PathBuf::default());
        assert!(settings.is_implicitly_active("Update.esm"));
        assert!(!settings.is_implicitly_active("Test.esm"));
    }

    #[test]
    fn plugins_folder_should_be_a_child_of_the_game_path() {
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, Path::new("game"), &PathBuf::default());
        assert_eq!(Path::new("game/Data"), settings.plugins_folder());
    }

    #[test]
    fn load_order_file_should_be_in_local_path_for_skyrim_and_none_for_other_games() {
        let mut settings =
            GameSettings::with_local_path(GameId::Skyrim, Path::new("game"), Path::new("local"));
        assert_eq!(
            Path::new("local/loadorder.txt"),
            settings.load_order_file().as_ref().unwrap()
        );

        settings =
            GameSettings::with_local_path(GameId::SkyrimSE, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Morrowind, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Oblivion, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Fallout3, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::FalloutNV, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());

        settings =
            GameSettings::with_local_path(GameId::Fallout4, Path::new("game"), Path::new("local"));
        assert!(settings.load_order_file().is_none());
    }

    #[test]
    fn use_my_games_directory_should_be_false_if_the_ini_path_does_not_exist() {
        assert!(!use_my_games_directory(Path::new("does_not_exist")));
    }

    #[test]
    fn use_my_games_directory_should_be_false_if_the_ini_setting_value_is_not_1() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");
        let mut file = File::create(&ini_path).unwrap();
        file.write_all("...\nbUseMyGamesDirectory=0\n...".as_bytes())
            .unwrap();

        assert!(!use_my_games_directory(&ini_path));
    }

    #[test]
    fn use_my_games_directory_should_be_true_if_the_ini_setting_value_is_1() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");
        let mut file = File::create(&ini_path).unwrap();
        file.write_all("...\nbUseMyGamesDirectory=1\n...".as_bytes())
            .unwrap();

        assert!(use_my_games_directory(&ini_path));
    }
}
