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

use std::fs::rename;
use std::path::{Path, PathBuf};

use crate::enums::Error;

pub const GHOST_FILE_EXTENSION: &str = ".ghost";

pub trait GhostablePath {
    fn unghost(&self) -> Result<PathBuf, Error>;

    fn has_ghost_extension(&self) -> bool;

    fn resolve_path(&self) -> Result<PathBuf, Error>;
    fn as_ghosted_path(&self) -> Result<PathBuf, Error>;
    fn as_unghosted_path(&self) -> Result<PathBuf, Error>;
}

impl GhostablePath for Path {
    fn unghost(&self) -> Result<PathBuf, Error> {
        if !self.has_ghost_extension() {
            Ok(self.to_path_buf())
        } else {
            let new_path = self.as_unghosted_path()?;
            rename(self, &new_path).map_err(|e| Error::IoError(self.to_path_buf(), e))?;
            Ok(new_path)
        }
    }

    fn has_ghost_extension(&self) -> bool {
        match self.extension() {
            None => false,
            Some(x) => x == "ghost",
        }
    }

    fn resolve_path(&self) -> Result<PathBuf, Error> {
        if self.exists() {
            Ok(self.to_path_buf())
        } else {
            let alt_path = if self.has_ghost_extension() {
                self.as_unghosted_path()?
            } else {
                self.as_ghosted_path()?
            };

            if alt_path.exists() {
                Ok(alt_path)
            } else {
                Err(Error::InvalidPath(self.to_path_buf()))
            }
        }
    }

    fn as_ghosted_path(&self) -> Result<PathBuf, Error> {
        if self.has_ghost_extension() {
            Ok(self.to_path_buf())
        } else {
            self.file_name()
                .ok_or_else(|| Error::NoFilename(self.to_path_buf()))
                .map(|x| {
                    let mut filename = x.to_os_string();
                    filename.push(GHOST_FILE_EXTENSION);

                    self.with_file_name(filename)
                })
        }
    }

    fn as_unghosted_path(&self) -> Result<PathBuf, Error> {
        if !self.has_ghost_extension() {
            Ok(self.to_path_buf())
        } else {
            self.file_stem()
                .map(|f| self.with_file_name(f))
                .ok_or_else(|| Error::NoFilename(self.to_path_buf()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{copy, create_dir};
    use tempfile::tempdir;

    fn copy_to_test_dir(from_file: &str, to_file: &str, game_dir: &Path) {
        let testing_plugins_dir = Path::new("testing-plugins/Oblivion/Data");
        let data_dir = game_dir.join("Data");
        if !data_dir.exists() {
            create_dir(&data_dir).unwrap();
        }
        copy(testing_plugins_dir.join(from_file), data_dir.join(to_file)).unwrap();
    }

    #[test]
    fn unghost_should_rename_the_path_with_no_ghost_extension() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", game_dir);
        let expected_path = data_dir.join("Blank.esp");
        let ghosted_path = data_dir.join("Blank.esp.ghost").unghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn unghost_should_do_nothing_if_the_path_is_already_unghosted() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp", game_dir);
        let expected_path = data_dir.join("Blank.esp");
        let ghosted_path = expected_path.unghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn has_ghost_extension_should_be_true_iff_path_ends_in_dot_ghost() {
        assert!(Path::new("Data/plugin.esp.ghost").has_ghost_extension());
        assert!(!Path::new("Data/plugin.esp").has_ghost_extension());
        assert!(!Path::new("Data/plugin").has_ghost_extension());
    }

    #[test]
    fn as_ghosted_path_should_return_given_path_if_it_ends_in_dot_ghost() {
        let path = Path::new("Data/plugin.esp.ghost");
        let ghosted_path = path.as_ghosted_path().unwrap();

        assert_eq!(path, ghosted_path);
    }

    #[test]
    fn as_ghosted_path_should_return_path_with_dot_ghost_extension() {
        let path = Path::new("Data/plugin.esp");
        let ghosted_path = path.as_ghosted_path().unwrap();

        assert_eq!(Path::new("Data/plugin.esp.ghost"), ghosted_path);
    }

    #[test]
    fn as_ghosted_path_should_error_if_the_given_path_does_not_have_a_filename() {
        assert!(Path::new("/").as_ghosted_path().is_err());
    }

    #[test]
    fn as_unghosted_path_should_return_path_unchanged_if_it_does_not_end_in_dot_ghost() {
        let path = Path::new("Data/plugin.esp");
        let unghosted_path = path.as_unghosted_path().unwrap();

        assert_eq!(Path::new("Data/plugin.esp"), unghosted_path);
    }

    #[test]
    fn as_unghosted_path_should_remove_dot_ghost_suffix() {
        let path = Path::new("Data/plugin.esp.ghost");
        let unghosted_path = path.as_unghosted_path().unwrap();

        assert_eq!(Path::new("Data/plugin.esp"), unghosted_path);
    }

    #[test]
    fn resolve_path_should_return_the_path_that_exists() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp", game_dir);
        let mut expected_path = data_dir.join("Blank.esp");
        let mut resolved_path = data_dir.join("Blank.esp.ghost").resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);

        resolved_path = expected_path.resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", game_dir);
        expected_path = data_dir.join("Blank.esm.ghost");
        resolved_path = data_dir.join("Blank.esm").resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);
    }

    #[test]
    fn resolve_path_should_error_if_no_path_exists() {
        assert!(Path::new("foo").resolve_path().is_err());
    }
}
