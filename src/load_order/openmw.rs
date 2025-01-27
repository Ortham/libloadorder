use crate::{plugin::Plugin, Error, GameSettings};

use super::{
    mutable::MutableLoadOrder,
    readable::ReadableLoadOrderBase,
    writable::{activate, add, deactivate, remove, set_active_plugins},
    WritableLoadOrder,
};

#[derive(Clone, Debug)]
pub struct OpenMWLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl OpenMWLoadOrder {
    pub fn new(game_settings: GameSettings) -> Self {
        Self {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn read_from_active_plugins_file(&self) -> Result<Vec<(String, bool)>, Error> {
        todo!()
    }
}

impl ReadableLoadOrderBase for OpenMWLoadOrder {
    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    fn game_settings_base(&self) -> &GameSettings {
        &self.game_settings
    }
}

impl MutableLoadOrder for OpenMWLoadOrder {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for OpenMWLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings {
        &mut self.game_settings
    }

    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        let plugin_tuples = self.read_from_active_plugins_file()?;
        let paths = self.find_plugins();

        self.load_unique_plugins(plugin_tuples, paths);

        self.add_implicitly_active_plugins()?;

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn add(&mut self, plugin_name: &str) -> Result<usize, Error> {
        add(self, plugin_name)
    }

    fn remove(&mut self, plugin_name: &str) -> Result<(), Error> {
        remove(self, plugin_name)
    }

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        self.replace_plugins(plugin_names)
    }

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<usize, Error> {
        MutableLoadOrder::set_plugin_index(self, plugin_name, position)
    }

    fn is_self_consistent(&self) -> Result<bool, Error> {
        Ok(true)
    }

    fn is_ambiguous(&self) -> Result<bool, Error> {
        Ok(false)
    }

    fn activate(&mut self, plugin_name: &str) -> Result<(), Error> {
        activate(self, plugin_name)
    }

    fn deactivate(&mut self, plugin_name: &str) -> Result<(), Error> {
        deactivate(self, plugin_name)
    }

    fn set_active_plugins(&mut self, active_plugin_names: &[&str]) -> Result<(), Error> {
        set_active_plugins(self, active_plugin_names)
    }
}

#[cfg(test)]
mod tests {}
