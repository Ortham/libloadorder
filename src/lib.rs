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

mod enums;
mod game_settings;
mod ghostable_path;
mod ini;
mod load_order;
mod plugin;
#[cfg(test)]
mod tests;

pub use crate::enums::{Error, GameId, LoadOrderMethod};
pub use crate::game_settings::GameSettings;
pub use crate::load_order::{ReadableLoadOrder, WritableLoadOrder};

fn is_enderal(game_path: &std::path::Path) -> bool {
    game_path.join("Enderal Launcher.exe").exists()
}
