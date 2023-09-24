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
    let contents = std::fs::read(ini_path)?;

    // My Games is used if bUseMyGamesDirectory is not present or set to 1.
    let contents = WINDOWS_1252.decode_without_bom_handling(&contents).0;

    ini::Ini::load_from_str(&contents).map_err(Error::from)
}

pub fn use_my_games_directory(ini_path: &Path) -> Result<bool, Error> {
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

fn merge_test_files(mut base: TestFiles, overrider: TestFiles) -> TestFiles {
    for i in 0..10 {
        if overrider[i].is_some() {
            base[i] = overrider[i].clone();
        }
    }

    base
}

fn filter_test_files(test_files: TestFiles) -> Vec<String> {
    IntoIterator::into_iter(test_files)
        .flatten()
        .filter(|e| !e.is_empty())
        .collect()
}

pub fn test_files(
    game_id: GameId,
    game_path: &Path,
    my_games_path: &Path,
) -> Result<Vec<String>, Error> {
    match game_id {
        GameId::Morrowind => Ok(Vec::new()),
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
            let test_files = merge_test_files(base_test_files, custom_test_files);
            Ok(filter_test_files(test_files))
        }
        GameId::Fallout4VR => {
            let base_test_files = read_test_files(&my_games_path.join("Fallout4VR.ini"))?;
            let custom_test_files = read_test_files(&my_games_path.join("Fallout4VRCustom.ini"))?;
            let test_files = merge_test_files(base_test_files, custom_test_files);
            Ok(filter_test_files(test_files))
        }
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

        files1[0] = Some("a".to_string());

        files2[1] = Some("b".to_string());
        files2[2] = Some("c".to_string());
        files2[3] = Some("d".to_string());
        files2[4] = Some("e".to_string());
        files2[5] = Some("f".to_string());
        files2[6] = Some("g".to_string());
        files2[7] = Some("h".to_string());
        files2[8] = Some("i".to_string());
        files2[9] = Some("j".to_string());

        let output = merge_test_files(files1.clone(), files2.clone());

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

        files2[0] = Some("aa".to_string());

        let output = merge_test_files(files1, files2);

        assert_eq!("aa", output[0].as_ref().unwrap());
    }

    #[test]
    fn filter_test_files_should_remove_none_and_empty_string_values() {
        let mut files = TestFiles::default();

        files[0] = Some("a".to_string());
        files[1] = None;
        files[2] = Some("c".to_string());
        files[3] = Some(String::new());
        files[4] = Some("e".to_string());

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
}
