/*  libloadorder

A library for reading and writing the load order of plugin files for
TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
Fallout: New Vegas.

Copyright (C) 2015    WrinklyNinja

This file is part of libloadorder.

libloadorder is free software: you can redistribute
it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

libloadorder is distributed in the hope that it will
be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with libloadorder.  If not, see
<http://www.gnu.org/licenses/>.
*/

#include <gtest/gtest.h>

#include "backend/helpers.h"

namespace liblo {
    namespace test {
        TEST(copyString, shouldCreateACopyOfTheInputString) {
            char * cstring = nullptr;

            cstring = copyString("temporary string");

            EXPECT_STREQ("temporary string", cstring);

            delete[] cstring;
        }

        TEST(fileToBuffer, shouldThrowIfFileDoesntExist) {
            EXPECT_ANY_THROW(fileToBuffer("missing"));
        }

        TEST(fileToBuffer, shouldReadFileContentsIfItExists) {
            boost::filesystem::path file = "./test.tmp";
            std::string expectedFileContent = "this is a test file,\nit has two lines of text.";

            boost::filesystem::ofstream out(file);
            out << expectedFileContent;
            out.close();

            EXPECT_EQ(expectedFileContent, fileToBuffer(file));

            ASSERT_NO_THROW(boost::filesystem::remove(file));
        }

        TEST(windows1252toUtf8, shouldConvertWindows1252EncodedTextToUtf8EncodedText) {
            std::string inputWindows1252 = "T\xE8st";
            std::string expectedUtf8 = "T\xC3\xA8st";

            EXPECT_EQ(expectedUtf8, windows1252toUtf8(inputWindows1252));
        }
#ifdef _WIN32
        TEST(windows1252toUtf8, shouldNotThrowIfInputStringContainsBytesThatAreInvalidInWindows1252) {
            EXPECT_NO_THROW(windows1252toUtf8("\x81\x8D\x8F\x90\x9D"));
        }
#else
        TEST(windows1252toUtf8, shouldThrowIfInputStringContainsBytesThatAreInvalidInWindows1252) {
            EXPECT_ANY_THROW(windows1252toUtf8("\x81\x8D\x8F\x90\x9D"));
        }
#endif

        TEST(utf8ToWindows1252, shouldConvertUtf8EncodedTextToWindows1252EncodedTextIfAllCharactersCanBeRepresented) {
            std::string inputUtf8 = "T\xC3\xA8st";
            std::string expectedWindows1252 = "T\xE8st";

            EXPECT_EQ(expectedWindows1252, utf8ToWindows1252(inputUtf8));
        }

        TEST(utf8ToWindows1252, shouldThrowIfTheInputTextContainsCharactersThatCannotBeRepresentedInWindows1252) {
            EXPECT_ANY_THROW(utf8ToWindows1252("\xD0\xA0\xD1\x83\xD1\x81\xD1\x81\xD0\xBA\xD0\xB8\xD0\xB9"));
        }

        TEST(utf8ToWindows1252, shouldThrowIfInputStringContainsBytesThatAreInvalidInUtf8) {
            EXPECT_ANY_THROW(utf8ToWindows1252("\xC0"));
        }
    }
}
