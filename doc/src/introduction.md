# An Introduction To Load Orders

This documentation provides a general overview of load ordering in the following games:

- TES III: Morrowind
- TES IV: Oblivion
- TES V: Skyrim
- TES V: Skyrim Special Edition
- Fallout 3
- Fallout: New Vegas
- Fallout 4

For simplicity, "the game" will be used when the text refers to any of the games in the previous list.

Mod plugins for the game are files that end in `.esp` or `.esm`. These files are created by the game's official editing tools, or by third party modders' tools. They contain various data entries, which can either be brand new additions or changes to the entries added by another plugin, including the game's main master file (`Morrowind.esm`, `Oblivion.esm`, `Skyrim.esm`, `Fallout3.esm`, `FalloutNV.esm` or `Fallout4.esm` depending on the game). These entries cover almost all aspects of what is in the game, eg. NPCs, items, races, interiors, worlds, quests, etc.

When the game is run, it loads each of the plugins you have installed one by one. The order in which it loads them is, unsurprisingly, what is referred to as the load order. The load order is important for two reasons:

- Dependency resolution.
- Conflict resolution.

A plugin which changes the entries added by other plugins is dependent on those other plugins, and the game requires all of the latter plugins to be loaded before the former. If they aren't, the game will crash on launch.

## Displaying the Load Order

A plugin's position in the load order is often displayed by mod managers as a hexadecimal number, from 00 to FE (0 to 255 in decimal). The plugin with position 00 loads first, and the plugin with position FE is loaded last. Hexadecimal numbers are used to display the load order positions of plugins because these numbers form the first two digits of the code that the game uses to reference the entries that the plugin adds, so knowing the numbers allows modders and mod users to determine which plugin an entry is from.
