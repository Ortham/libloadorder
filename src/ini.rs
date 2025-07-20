/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2023 Oliver Hamlet
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

use std::path::Path;

use encoding_rs::WINDOWS_1252;

use crate::{Error, GameId};

type TestFiles = [Option<String>; 10];

fn read_ini(ini_path: &Path) -> Result<ini::Ini, Error> {
    // Read ini as Windows-1252 bytes and then convert to UTF-8 before parsing,
    // as the ini crate expects the content to be valid UTF-8.
    let contents =
        std::fs::read(ini_path).map_err(|e| Error::IoError(ini_path.to_path_buf(), e))?;

    // My Games is used if bUseMyGamesDirectory is not present or set to 1.
    let contents = WINDOWS_1252.decode_without_bom_handling(&contents).0;

    ini::Ini::load_from_str_opt(
        &contents,
        ini::ParseOption {
            enabled_quote: false,
            enabled_escape: false,
            ..ini::ParseOption::default()
        },
    )
    .map_err(|e| Error::IniParsingError {
        path: ini_path.to_path_buf(),
        line: e.line,
        column: e.col,
        message: e.msg.to_string(),
    })
}

pub(crate) fn use_my_games_directory(ini_path: &Path) -> Result<bool, Error> {
    if ini_path.exists() {
        // My Games is used if bUseMyGamesDirectory is not present or set to 1.
        read_ini(ini_path)
            .map(|ini| ini.get_from(Some("General"), "bUseMyGamesDirectory") != Some("0"))
    } else {
        Ok(true)
    }
}

fn read_test_files(ini_path: &Path) -> Result<TestFiles, Error> {
    if !ini_path.exists() {
        return Ok(TestFiles::default());
    }

    let ini = read_ini(ini_path)?;

    let mut test_files = TestFiles::default();

    test_files[0] = ini
        .get_from(Some("General"), "sTestFile1")
        .map(ToString::to_string);
    test_files[1] = ini
        .get_from(Some("General"), "sTestFile2")
        .map(ToString::to_string);
    test_files[2] = ini
        .get_from(Some("General"), "sTestFile3")
        .map(ToString::to_string);
    test_files[3] = ini
        .get_from(Some("General"), "sTestFile4")
        .map(ToString::to_string);
    test_files[4] = ini
        .get_from(Some("General"), "sTestFile5")
        .map(ToString::to_string);
    test_files[5] = ini
        .get_from(Some("General"), "sTestFile6")
        .map(ToString::to_string);
    test_files[6] = ini
        .get_from(Some("General"), "sTestFile7")
        .map(ToString::to_string);
    test_files[7] = ini
        .get_from(Some("General"), "sTestFile8")
        .map(ToString::to_string);
    test_files[8] = ini
        .get_from(Some("General"), "sTestFile9")
        .map(ToString::to_string);
    test_files[9] = ini
        .get_from(Some("General"), "sTestFile10")
        .map(ToString::to_string);

    Ok(test_files)
}

fn merge_test_files(mut base: TestFiles, overrider: &TestFiles) -> TestFiles {
    base.iter_mut().zip(overrider.iter()).for_each(|(b, o)| {
        if o.is_some() {
            b.clone_from(o);
        }
    });

    base
}

fn filter_test_files(test_files: TestFiles) -> Vec<String> {
    IntoIterator::into_iter(test_files)
        .flatten()
        .filter(|e| !e.is_empty())
        .collect()
}

pub(crate) fn test_files(
    game_id: GameId,
    game_path: &Path,
    my_games_path: &Path,
) -> Result<Vec<String>, Error> {
    match game_id {
        GameId::Morrowind | GameId::OpenMW | GameId::OblivionRemastered => Ok(Vec::new()),
        GameId::Oblivion => {
            let ini_path = game_path.join("Oblivion.ini");

            let ini_path = if use_my_games_directory(&ini_path)? {
                my_games_path.join("Oblivion.ini")
            } else {
                ini_path
            };

            let test_files = read_test_files(&ini_path)?;
            Ok(filter_test_files(test_files))
        }
        GameId::Skyrim | GameId::SkyrimSE => {
            let filename = if crate::is_enderal(game_path) {
                "Enderal.ini"
            } else {
                "Skyrim.ini"
            };

            let test_files = read_test_files(&my_games_path.join(filename))?;
            Ok(filter_test_files(test_files))
        }
        GameId::SkyrimVR => {
            let test_files = read_test_files(&my_games_path.join("SkyrimVR.ini"))?;
            Ok(filter_test_files(test_files))
        }
        GameId::Fallout3 => {
            let test_files = read_test_files(&my_games_path.join("FALLOUT.INI"))?;
            Ok(filter_test_files(test_files))
        }
        GameId::FalloutNV => {
            let test_files = read_test_files(&my_games_path.join("Fallout.ini"))?;
            Ok(filter_test_files(test_files))
        }
        GameId::Fallout4 => {
            let base_test_files = read_test_files(&my_games_path.join("Fallout4.ini"))?;
            let custom_test_files = read_test_files(&my_games_path.join("Fallout4Custom.ini"))?;
            let test_files = merge_test_files(base_test_files, &custom_test_files);
            Ok(filter_test_files(test_files))
        }
        GameId::Fallout4VR => {
            let base_test_files = read_test_files(&my_games_path.join("Fallout4VR.ini"))?;
            let custom_test_files = read_test_files(&my_games_path.join("Fallout4VRCustom.ini"))?;
            let test_files = merge_test_files(base_test_files, &custom_test_files);
            Ok(filter_test_files(test_files))
        }
        GameId::Starfield => {
            let base_test_files = read_test_files(&game_path.join("Starfield.ini"))?;
            let custom_test_files = read_test_files(&my_games_path.join("StarfieldCustom.ini"))?;

            let language = starfield_language(game_path)?;
            let language_ini_path = my_games_path.join(format!("Starfield_{language}.INI"));
            let language_test_files = read_test_files(&language_ini_path)?;

            let test_files = merge_test_files(base_test_files, &language_test_files);
            let test_files = merge_test_files(test_files, &custom_test_files);
            Ok(filter_test_files(test_files))
        }
    }
}

fn starfield_language(game_path: &Path) -> Result<&'static str, Error> {
    let steam_acf_path = game_path.join("../../appmanifest_1716740.acf");

    let language = if steam_acf_path.exists() {
        // Steam install: Get language from app manifest's AppState.UserConfig.language.
        read_steam_language_config(&steam_acf_path)?.map(map_steam_language)
    } else {
        // Microsoft Store install: Get system language from Windows.
        get_windows_system_language()?.map(map_windows_system_language)
    };

    Ok(language.unwrap_or("en"))
}

fn read_steam_language_config(appmanifest_acf_path: &Path) -> Result<Option<String>, Error> {
    let content = std::fs::read_to_string(appmanifest_acf_path)
        .map_err(|e| Error::IoError(appmanifest_acf_path.to_path_buf(), e))?;

    let language = keyvalues_parser::Vdf::parse(&content)
        .map_err(|e| {
            let detail = match e {
                keyvalues_parser::error::Error::EscapedParseError(e) => e.to_string(),

                keyvalues_parser::error::Error::RawParseError(e) => e.to_string(),

                keyvalues_parser::error::Error::RenderError(e) => e.to_string(),
                keyvalues_parser::error::Error::RawRenderError { invalid_char } => {
                    format!("Invalid character \"{invalid_char}\"")
                }
            };

            Error::VdfParsingError(appmanifest_acf_path.to_path_buf(), detail)
        })?
        .value
        .get_obj()
        .and_then(|o| o.get("UserConfig"))
        .and_then(|v| v.first())
        .and_then(keyvalues_parser::Value::get_obj)
        .and_then(|o| o.get("language"))
        .and_then(|v| v.first())
        .and_then(keyvalues_parser::Value::get_str)
        .map(ToString::to_string);

    Ok(language)
}

fn map_steam_language<T: AsRef<str>>(steam_language: T) -> &'static str {
    match steam_language.as_ref() {
        "german" => "de",
        "french" => "fr",
        "italian" => "it",
        "spanish" | "latam" => "es",
        "schinese" => "zhhans",
        "japanese" => "ja",
        "polish" => "pl",
        "brazilian" => "ptbr",
        _ => "en",
    }
}

#[cfg(windows)]
fn get_windows_system_language() -> Result<Option<String>, Error> {
    // Languages are BCP-47 language tags, e.g. "en-GB", "es-AR", "fr-FR"
    // to_string_lossy() is fine here because language tags are ASCII-only.
    let language = windows::System::UserProfile::GlobalizationPreferences::Languages()?
        .into_iter()
        .next()
        .map(|l| l.to_string_lossy());

    Ok(language)
}

#[cfg(not(windows))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "To match the Windows function signature"
)]
fn get_windows_system_language() -> Result<Option<String>, Error> {
    Ok(None)
}

fn map_windows_system_language<T: AsRef<str>>(system_language: T) -> &'static str {
    let mut parts = system_language.as_ref().split('-');

    match parts.next().unwrap_or("en") {
        // These language values are listed in Starfield's MicrosoftGame.Config and
        // appxmanifest.xml.
        "fr" => "fr",
        "es" => "es",
        "pl" => "pl",
        "de" => "de",
        "it" => "it",
        "ja" => "ja",
        "pt" => {
            if let Some("BR") = parts.next() {
                "ptbr"
            } else {
                "en"
            }
        }
        "zh" => {
            if let Some("Hans") = parts.next() {
                "zhhans"
            } else {
                "en"
            }
        }
        _ => "en",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    fn prep_dirs(tempdir: &tempfile::TempDir) -> (PathBuf, PathBuf) {
        let game_path = tempdir.path().join("game");
        let my_games_path = tempdir.path().join("my games");

        std::fs::create_dir_all(&game_path).unwrap();
        std::fs::create_dir_all(&my_games_path).unwrap();

        (game_path, my_games_path)
    }

    #[test]
    fn starfield_language_should_use_steam_acf_if_it_exists() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("common").join("Starfield");
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState"
        {
            "UserConfig"
            {
                "language" "german"
            }
        }"#;

        std::fs::write(&app_manifest_path, content).unwrap();
        std::fs::create_dir_all(&game_path).unwrap();

        let language = starfield_language(&game_path).unwrap();

        assert_eq!("de", language);
    }

    #[cfg(windows)]
    #[test]
    fn starfield_language_should_return_en_if_steam_acf_does_not_specify_a_language() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("common").join("Starfield");
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState"{}"#;

        std::fs::write(&app_manifest_path, content).unwrap();

        let language = starfield_language(&game_path).unwrap();

        assert_eq!("en", language);
    }

    #[cfg(windows)]
    #[test]
    fn starfield_language_should_use_windows_language_if_steam_acf_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("common").join("Starfield");

        assert!(starfield_language(&game_path).is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn starfield_language_should_use_en_if_steam_acf_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("common").join("Starfield");

        assert_eq!("en", starfield_language(&game_path).unwrap());
    }

    #[test]
    fn read_steam_language_config_should_read_from_acf_file() {
        let tmp_dir = tempdir().unwrap();
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState"
        {
            "appid"		"1716740"
            "Universe"		"1"
            "LauncherPath"		"C:\\Program Files (x86)\\Steam\\steam.exe"
            "name"		"Starfield"
            "StateFlags"		"4"
            "installdir"		"Starfield"
            "LastUpdated"		"1695657094"
            "SizeOnDisk"		"131019631899"
            "StagingSize"		"0"
            "buildid"		"12212450"
            "LastOwner"		"76561198033938668"
            "UpdateResult"		"0"
            "BytesToDownload"		"6109484544"
            "BytesDownloaded"		"6109484544"
            "BytesToStage"		"6171029640"
            "BytesStaged"		"6171029640"
            "TargetBuildID"		"12212450"
            "AutoUpdateBehavior"		"0"
            "AllowOtherDownloadsWhileRunning"		"0"
            "ScheduledAutoUpdate"		"0"
            "StagingFolder"		"0"
            "InstalledDepots"
            {
                "1716741"
                {
                    "manifest"		"3276175983502685135"
                    "size"		"108496536"
                }
                "1716742"
                {
                    "manifest"		"7068708531301311719"
                    "size"		"124674684083"
                }
                "2401180"
                {
                    "manifest"		"1350450549444461803"
                    "size"		"50772412"
                    "dlcappid"		"2401180"
                }
                "2401181"
                {
                    "manifest"		"9009164480135609609"
                    "size"		"14760227"
                    "dlcappid"		"2401181"
                }
                "1716743"
                {
                    "manifest"		"1387979402837597913"
                    "size"		"6171029640"
                }
            }
            "SharedDepots"
            {
                "228989"		"228980"
                "228990"		"228980"
            }
            "UserConfig"
            {
                "language"		"german"
            }
            "MountedConfig"
            {
                "language"		"german"
            }
        }"#;

        std::fs::write(&app_manifest_path, content).unwrap();

        let language = read_steam_language_config(&app_manifest_path)
            .unwrap()
            .unwrap();

        assert_eq!("german", language);
    }

    #[test]
    fn read_steam_language_config_should_error_if_acf_file_does_not_exist() {
        let language = read_steam_language_config(&PathBuf::default());

        assert!(language.is_err());
    }

    #[test]
    fn read_steam_language_config_should_error_if_acf_file_has_no_content() {
        let tmp_dir = tempdir().unwrap();
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        std::fs::write(&app_manifest_path, "").unwrap();

        let language = read_steam_language_config(&app_manifest_path);

        assert!(language.is_err());
    }

    #[test]
    fn read_steam_language_config_should_error_if_acf_file_has_no_appstate_value() {
        let tmp_dir = tempdir().unwrap();
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState""#;

        std::fs::write(&app_manifest_path, content).unwrap();

        let language = read_steam_language_config(&app_manifest_path);

        assert!(language.is_err());
    }

    #[test]
    fn read_steam_language_config_should_return_none_if_acf_file_has_no_userconfig() {
        let tmp_dir = tempdir().unwrap();
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState"{}"#;

        std::fs::write(&app_manifest_path, content).unwrap();

        let language = read_steam_language_config(&app_manifest_path).unwrap();

        assert!(language.is_none());
    }

    #[test]
    fn read_steam_language_config_should_return_none_if_acf_file_has_no_language() {
        let tmp_dir = tempdir().unwrap();
        let app_manifest_path = tmp_dir.path().join("appmanifest_1716740.acf");

        let content = r#""AppState"
        {
            "UserConfig"
            {

            }
        }"#;

        std::fs::write(&app_manifest_path, content).unwrap();

        let language = read_steam_language_config(&app_manifest_path).unwrap();

        assert!(language.is_none());
    }

    #[test]
    fn map_steam_language_should_map_languages_supported_by_starfield() {
        assert_eq!("de", map_steam_language("german"));
        assert_eq!("fr", map_steam_language("french"));
        assert_eq!("it", map_steam_language("italian"));
        assert_eq!("es", map_steam_language("spanish"));
        assert_eq!("es", map_steam_language("latam"));
        assert_eq!("zhhans", map_steam_language("schinese"));
        assert_eq!("ja", map_steam_language("japanese"));
        assert_eq!("pl", map_steam_language("polish"));
        assert_eq!("ptbr", map_steam_language("brazilian"));
    }

    #[test]
    fn map_steam_language_should_map_unrecognised_languages_to_en() {
        assert_eq!("en", map_steam_language("russian"));
    }

    #[cfg(windows)]
    #[test]
    fn get_windows_system_language_should_return_a_non_empty_option() {
        let language = get_windows_system_language().unwrap();

        assert!(language.is_some());
        assert!(language.unwrap().len() >= 2);
    }

    #[cfg(not(windows))]
    #[test]
    fn get_windows_system_language_should_return_none() {
        assert!(get_windows_system_language().unwrap().is_none());
    }

    #[test]
    fn map_windows_system_language_should_map_languages_supported_by_starfield() {
        assert_eq!("en", map_windows_system_language("en-whatever"));
        assert_eq!("fr", map_windows_system_language("fr-whatever"));
        assert_eq!("es", map_windows_system_language("es-whatever"));
        assert_eq!("pl", map_windows_system_language("pl-whatever"));
        assert_eq!("de", map_windows_system_language("de-whatever"));
        assert_eq!("it", map_windows_system_language("it-whatever"));
        assert_eq!("ja", map_windows_system_language("ja-whatever"));
        assert_eq!("ptbr", map_windows_system_language("pt-BR"));
        assert_eq!("zhhans", map_windows_system_language("zh-Hans"));
    }

    #[test]
    fn map_windows_system_language_should_map_unrecognised_languages_to_en() {
        assert_eq!("en", map_steam_language("ru"));
        assert_eq!("en", map_steam_language("pt-PT"));
        assert_eq!("en", map_steam_language("zh-Hant"));
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

        assert_eq!(
            Some("\u{c0}.esp"),
            ini.get_from(Some("General"), "sTestFile1")
        );
    }

    #[test]
    fn read_ini_should_not_treat_quotes_as_special_characters() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(
            &ini_path,
            br#"[General]
            a="double quoted string"
            b=string "containing" double quotes
            c='single quoted string'
            d=string 'containing' double quotes
            e="mismatched double quote
            f='mismatched single quote
        "#,
        )
        .unwrap();

        let ini = read_ini(&ini_path).unwrap();

        assert_eq!(
            Some("\"double quoted string\""),
            ini.get_from(Some("General"), "a")
        );
        assert_eq!(
            Some("string \"containing\" double quotes"),
            ini.get_from(Some("General"), "b")
        );
        assert_eq!(
            Some("'single quoted string'"),
            ini.get_from(Some("General"), "c")
        );
        assert_eq!(
            Some("string 'containing' double quotes"),
            ini.get_from(Some("General"), "d")
        );
        assert_eq!(
            Some("\"mismatched double quote"),
            ini.get_from(Some("General"), "e")
        );
        assert_eq!(
            Some("'mismatched single quote"),
            ini.get_from(Some("General"), "f")
        );
    }

    #[test]
    fn read_ini_should_not_treat_backslash_as_an_escape_character() {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path();
        let ini_path = game_path.join("Oblivion.ini");

        std::fs::write(
            &ini_path,
            br"[General]
            a=\\\0\a\b\t\r\n\x20
            ",
        )
        .unwrap();

        let ini = read_ini(&ini_path).unwrap();

        assert_eq!(
            Some(r"\\\0\a\b\t\r\n\x20"),
            ini.get_from(Some("General"), "a")
        );
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

    #[test]
    fn read_test_files_should_return_array_of_nones_if_path_does_not_exist() {
        let test_files = read_test_files(Path::new("missing.ini")).unwrap();

        assert_eq!(10, test_files.len());
        assert!(test_files.iter().all(Option::is_none));
    }

    #[test]
    fn read_test_files_should_read_values_of_stestfile1_through_10() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(
            &ini_path,
            "[General]
            sTestFile1=a
            sTestFile2=b
            sTestFile3=c
            sTestFile4=d
            sTestFile5=e
            sTestFile6=f
            sTestFile7=g
            sTestFile8=h
            sTestFile9=i
            sTestFile10=j",
        )
        .unwrap();

        let test_files = read_test_files(&ini_path).unwrap();

        assert_eq!("a", test_files[0].as_ref().unwrap());
        assert_eq!("b", test_files[1].as_ref().unwrap());
        assert_eq!("c", test_files[2].as_ref().unwrap());
        assert_eq!("d", test_files[3].as_ref().unwrap());
        assert_eq!("e", test_files[4].as_ref().unwrap());
        assert_eq!("f", test_files[5].as_ref().unwrap());
        assert_eq!("g", test_files[6].as_ref().unwrap());
        assert_eq!("h", test_files[7].as_ref().unwrap());
        assert_eq!("i", test_files[8].as_ref().unwrap());
        assert_eq!("j", test_files[9].as_ref().unwrap());
    }

    #[test]
    fn read_test_files_should_ignore_values_outside_the_general_section() {
        let tmp_dir = tempdir().unwrap();
        let ini_path = tmp_dir.path().join("ini.ini");

        std::fs::write(
            &ini_path,
            "[General]
            sTestFile1=a
            [Other]
            sTestFile2=b",
        )
        .unwrap();

        let test_files = read_test_files(&ini_path).unwrap();

        assert_eq!("a", test_files[0].as_ref().unwrap());
        assert!(test_files[1].is_none());
    }

    #[test]
    fn merge_test_files_should_overwrite_values_if_the_overrider_value_is_not_none() {
        let mut files1 = TestFiles::default();
        let mut files2 = TestFiles::default();

        files1[0] = Some("a".to_owned());

        files2[1] = Some("b".to_owned());
        files2[2] = Some("c".to_owned());
        files2[3] = Some("d".to_owned());
        files2[4] = Some("e".to_owned());
        files2[5] = Some("f".to_owned());
        files2[6] = Some("g".to_owned());
        files2[7] = Some("h".to_owned());
        files2[8] = Some("i".to_owned());
        files2[9] = Some("j".to_owned());

        let output = merge_test_files(files1.clone(), &files2.clone());

        assert_eq!("a", output[0].as_ref().unwrap());
        assert_eq!("b", output[1].as_ref().unwrap());
        assert_eq!("c", output[2].as_ref().unwrap());
        assert_eq!("d", output[3].as_ref().unwrap());
        assert_eq!("e", output[4].as_ref().unwrap());
        assert_eq!("f", output[5].as_ref().unwrap());
        assert_eq!("g", output[6].as_ref().unwrap());
        assert_eq!("h", output[7].as_ref().unwrap());
        assert_eq!("i", output[8].as_ref().unwrap());
        assert_eq!("j", output[9].as_ref().unwrap());

        files2[0] = Some("aa".to_owned());

        let output = merge_test_files(files1, &files2);

        assert_eq!("aa", output[0].as_ref().unwrap());
    }

    #[test]
    fn filter_test_files_should_remove_none_and_empty_string_values() {
        let mut files = TestFiles::default();

        files[0] = Some("a".to_owned());
        files[1] = None;
        files[2] = Some("c".to_owned());
        files[3] = Some(String::new());
        files[4] = Some("e".to_owned());

        let output = filter_test_files(files);

        assert_eq!(vec!["a", "c", "e"], output);
    }

    #[test]
    fn test_files_for_morrowind_should_return_an_empty_vec() {
        let files =
            test_files(GameId::Morrowind, &PathBuf::default(), &PathBuf::default()).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_files_for_oblivion_should_read_from_game_path_ini_if_not_using_my_games_directory() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = game_path.join("Oblivion.ini");
        std::fs::write(&ini_path, "[General]\nbUseMyGamesDirectory=0\nsTestFile1=a").unwrap();

        let files = test_files(GameId::Oblivion, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_oblivion_should_read_from_my_games_ini_if_using_it() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Oblivion.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::Oblivion, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_skyrim_should_read_from_my_games_skyrim_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Skyrim.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::Skyrim, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_enderal_should_read_from_my_games_enderal_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        std::fs::write(game_path.join("Enderal Launcher.exe"), "").unwrap();

        let ini_path = my_games_path.join("Enderal.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::Skyrim, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_skyrimse_should_read_from_my_games_skyrim_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Skyrim.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::SkyrimSE, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_enderalse_should_read_from_my_games_enderal_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        std::fs::write(game_path.join("Enderal Launcher.exe"), "").unwrap();

        let ini_path = my_games_path.join("Enderal.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::SkyrimSE, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_skyrimvr_should_read_from_my_games_skyrimvr_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("SkyrimVR.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::SkyrimVR, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_fallout3_should_read_from_my_games_fallout_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("FALLOUT.INI");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::Fallout3, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_falloutnv_should_read_from_my_games_fallout_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Fallout.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let files = test_files(GameId::FalloutNV, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a"], files);
    }

    #[test]
    fn test_files_for_fallout4_should_merge_values_from_my_games_fallout4_and_fallout4custom_ini() {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Fallout4.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let ini_path = my_games_path.join("Fallout4Custom.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile2=b").unwrap();

        let files = test_files(GameId::Fallout4, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a", "b"], files);
    }

    #[test]
    fn test_files_for_fallout4vr_should_merge_values_from_my_games_fallout4vr_and_fallout4vrcustom_ini(
    ) {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = my_games_path.join("Fallout4VR.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let ini_path = my_games_path.join("Fallout4VRCustom.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile2=b").unwrap();

        let files = test_files(GameId::Fallout4VR, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a", "b"], files);
    }

    #[test]
    fn test_files_for_starfield_should_merge_values_from_starfield_and_my_games_starfieldcustom_ini(
    ) {
        let tmp_dir = tempdir().unwrap();
        let (game_path, my_games_path) = prep_dirs(&tmp_dir);

        let ini_path = game_path.join("Starfield.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let ini_path = my_games_path.join("StarfieldCustom.ini");
        std::fs::write(&ini_path, "[General]\nsTestFile2=b").unwrap();

        let files = test_files(GameId::Starfield, &game_path, &my_games_path).unwrap();

        assert_eq!(vec!["a", "b"], files);
    }
}
