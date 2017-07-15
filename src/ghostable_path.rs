use std::fs::rename;
use std::path::{Path, PathBuf};
use error::Error;

pub trait GhostablePath {
    fn ghost(&self) -> Result<PathBuf, Error>;
    fn unghost(&self) -> Result<PathBuf, Error>;

    fn is_ghosted(&self) -> bool;

    fn resolve_path(&self) -> Result<PathBuf, Error>;
    fn as_ghosted_path(&self) -> Result<PathBuf, Error>;
    fn as_unghosted_path(&self) -> Result<PathBuf, Error>;
}

impl GhostablePath for Path {
    fn ghost(&self) -> Result<PathBuf, Error> {
        if self.is_ghosted() {
            Ok(self.to_path_buf())
        } else {
            let new_path = self.as_ghosted_path()?;
            rename(self, &new_path)?;
            Ok(new_path)
        }
    }

    fn unghost(&self) -> Result<PathBuf, Error> {
        if !self.is_ghosted() {
            Ok(self.to_path_buf())
        } else {
            let new_path = self.as_unghosted_path()?;
            rename(self, &new_path)?;
            Ok(new_path)
        }
    }

    fn is_ghosted(&self) -> bool {
        match self.extension() {
            None => false,
            Some(x) => x == "ghost",
        }
    }

    fn resolve_path(&self) -> Result<PathBuf, Error> {
        if self.exists() {
            Ok(self.to_path_buf())
        } else {
            let alt_path = if self.is_ghosted() {
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
        if self.is_ghosted() {
            Ok(self.to_path_buf())
        } else {
            self.file_name()
                .ok_or(Error::InvalidPath(self.to_path_buf()))
                .map(|x| {
                    let mut filename = x.to_os_string();
                    filename.push(".ghost");

                    self.with_file_name(filename)
                })
        }
    }

    fn as_unghosted_path(&self) -> Result<PathBuf, Error> {
        if !self.is_ghosted() {
            Ok(self.to_path_buf())
        } else {
            self.file_stem().map(|f| self.with_file_name(f)).ok_or(
                Error::InvalidPath(self.to_path_buf()),
            )
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;

    use std::fs::{copy, create_dir};
    use self::tempdir::TempDir;

    fn copy_to_test_dir(from_file: &str, to_file: &str, game_dir: &Path) {
        let testing_plugins_dir = Path::new("./tests/testing-plugins/Oblivion/Data");
        let data_dir = game_dir.join("Data");
        if !data_dir.exists() {
            create_dir(&data_dir).unwrap();
        }
        copy(testing_plugins_dir.join(from_file), data_dir.join(to_file)).unwrap();
    }

    #[test]
    fn ghost_should_rename_the_path_with_a_ghost_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let expected_path = data_dir.join("Blank.esp.ghost");
        let ghosted_path = data_dir.join("Blank.esp").ghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn ghost_should_do_nothing_if_the_path_is_already_ghosted() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &game_dir);
        let expected_path = data_dir.join("Blank.esp.ghost");
        let ghosted_path = expected_path.ghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn unghost_should_rename_the_path_with_no_ghost_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &game_dir);
        let expected_path = data_dir.join("Blank.esp");
        let ghosted_path = data_dir.join("Blank.esp.ghost").unghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn unghost_should_do_nothing_if_the_path_is_already_unghosted() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let expected_path = data_dir.join("Blank.esp");
        let ghosted_path = expected_path.unghost().unwrap();

        assert!(ghosted_path.exists());
        assert_eq!(expected_path, ghosted_path);
    }

    #[test]
    fn is_ghosted_should_be_true_iff_path_ends_in_dot_ghost() {
        assert!(Path::new("Data/plugin.esp.ghost").is_ghosted());
        assert!(!Path::new("Data/plugin.esp").is_ghosted());
        assert!(!Path::new("Data/plugin").is_ghosted());
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
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();
        let data_dir = game_dir.join("Data");

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let mut expected_path = data_dir.join("Blank.esp");
        let mut resolved_path = data_dir.join("Blank.esp.ghost").resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);

        resolved_path = expected_path.resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &game_dir);
        expected_path = data_dir.join("Blank.esm.ghost");
        resolved_path = data_dir.join("Blank.esm").resolve_path().unwrap();

        assert!(resolved_path.exists());
        assert_eq!(expected_path, resolved_path);
    }

    #[test]
    fn resolve_path_should_error_if_no_path_exists() {
        let mut path = Path::new("foo").resolve_path().is_err();
    }
}
