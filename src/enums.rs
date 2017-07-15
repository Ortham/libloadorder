/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libespm is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libespm is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libespm. If not, see <http://www.gnu.org/licenses/>.
 */

use espm::GameId as EspmId;

#[derive(Debug, PartialEq)]
pub enum LoadOrderMethod {
    Timestamp,
    Textfile,
    Asterisk,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameId {
    Morrowind,
    Oblivion,
    Skyrim,
    Fallout3,
    FalloutNV,
    Fallout4,
    SkyrimSE,
}

impl GameId {
    pub fn to_libespm_id(&self) -> EspmId {
        match *self {
            GameId::Morrowind => EspmId::Morrowind,
            GameId::Oblivion => EspmId::Oblivion,
            GameId::Skyrim => EspmId::Skyrim,
            GameId::Fallout3 => EspmId::Fallout3,
            GameId::FalloutNV => EspmId::FalloutNV,
            GameId::Fallout4 => EspmId::Fallout4,
            GameId::SkyrimSE => EspmId::Skyrim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_id_should_map_to_libespm_id_correctly() {
        assert_eq!(EspmId::Morrowind, GameId::Morrowind.to_libespm_id());
        assert_eq!(EspmId::Oblivion, GameId::Oblivion.to_libespm_id());
        assert_eq!(EspmId::Skyrim, GameId::Skyrim.to_libespm_id());
        assert_eq!(EspmId::Skyrim, GameId::SkyrimSE.to_libespm_id());
        assert_eq!(EspmId::Fallout3, GameId::Fallout3.to_libespm_id());
        assert_eq!(EspmId::FalloutNV, GameId::FalloutNV.to_libespm_id());
        assert_eq!(EspmId::Fallout4, GameId::Fallout4.to_libespm_id());
    }
}
