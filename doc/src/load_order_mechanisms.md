
## Load Order Mechanisms

So how does the game decide on the load order to use? Well, there are currently two different methods used, depending on the game (and its version):

- In Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas, load order is decided by the relative timestamps of plugins in the game's Data directory. An installed plugin's load order is therefore an intrinsic property of that plugin. The active plugin with the earliest date loads first, with plugins being listed in descending date order (earliest to latest).
- In all the editions of Skyrim and Fallout 4, load order is decide by the order in which plugins are listed in `plugins.txt`. This brought with it a fundamental change, in that load order position is no longer an intrinsic property of a plugin.

There are a few hardcoded rules that trump load order, irrespective of the method used to determine it:

- Master plugins almost always load before non-master plugins. The only exception is when a master plugin depends on a non-master plugin, in which case the non-master plugin will load between the earliest master plugin that depends on it and the preceding master plugin.
- In Skyrim:
  - `Skyrim.esm` always loads before all other plugins.
  - If `Update.esm` is present, it is always loaded. If it has no load order position set, it loads after all other master plugins. That might sound a bit odd, but recall that in Skyrim and Fallout 4 load order position is not an intrinsic property of a plugin.
- In Skyrim Special Edition, Skyrim VR, Fallout 4 and Fallout 4 VR:
  - Official Bethesda plugins are all hardcoded to always load in a certain order before other plugins. These include the game's main master file (`Skyrim.esm` or `Fallout4.esm`), DLC plugins (e.g. `Dragonborn.esm`, `DLCNukaWorld.esm`) and Creation Club plugins (e.g. `ccQDRSSE001-SurvivalMode.esl`).
  - Light plugins load amongst other plugins, but they all then get moved to the end of the load order so that in-game they all take up the `FE` load order slot. This movement does not affect conflict resolution.

A plugin's type is usually indicated by its file extension, for example `.esm` is used by master plugins, `.esl` is used by light plugins and `.esp` is used by non-master, non-light (i.e. normal) plugins. However, this isn't necessarily accurate, see Plugin Types for details.
