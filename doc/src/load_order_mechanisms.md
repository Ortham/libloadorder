
## Load Order Mechanisms

So how does the game decide on the load order to use? Well, there are currently two different methods used, depending on the game (and its version):

- In Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas, load order is decided by the relative timestamps of plugins in the game's Data directory. An installed plugin's load order is therefore an intrinsic property of that plugin. The active plugin with the earliest date loads first, with plugins being listed in descending date order (earliest to latest).
- In Skyrim and Fallout 4, load order is decide by the order in which plugins are listed in `plugins.txt`. This brought with it a fundamental change, in that load order position is no longer an intrinsic property of a plugin.

There are a few hardcoded rules that trump load order, irrespective of the method used to determine it:

- Master plugins always load before non-master plugins.
- In Skyrim and Fallout 4, the game's main master file (`Skyrim.esm` or `Fallout4.esm` respectively) always loads before all other plugins.
- In Skyrim, if `Update.esm` is present, it is always loaded. If it has no load order position set, it loads after all other master plugins. That might sound a bit odd, but recall that in Skyrim and Fallout 4 load order position is not an intrinsic property of a plugin.
