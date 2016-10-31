******************************
An Introduction To Load Orders
******************************

This section provides a general overview of load ordering in TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 Fallout: New Vegas and Fallout 4 for those who are unfamiliar with the concept. For simplicity, "the game" will be used when the text refers to any of the games in the previous list.

Mod plugins for the game are files that end in ``.esp`` or ``.esm``. These files are created by the game's official editing tools, or by third party modders' tools. They contain various data entries, which can either be brand new additions or changes to the entries added by another plugin, including the game's main master file (Morrowind.esm, Oblivion.esm, Skyrim.esm, Fallout3.esm, FalloutNV.esm or Fallout4.esm depending on the game). These entries cover almost all aspects of what is in the game, eg. NPCs, items, races, interiors, worlds, quests, etc.

When the game is run, it loads each of the plugins you have installed one by one. The order in which it loads them is, unsurprisingly, what is referred to as the load order. The load order is important for two reasons:

- Dependency resolution.
- Conflict resolution.

A plugin which changes the entries added by other plugins is dependent on those other plugins, and the game requires all of the latter plugins to be loaded before the former. If they aren't, the game will crash on launch.

The game can only apply one version of any one entry (this is known as the *rule of one*). Therefore, when more than one plugin changes the same entry, the load order is used to decide which plugin's changes are applied. Put simply, the last loaded plugin's changes are applied. The one exception to the rule of one is that the contents of cells (the physical space in which your character moves) can be changed by any number of plugins, so different plugins can add objects to cells all will see their changes applied by the game.

By changing the load order, you can change which plugins override and which are overridden, and so reduce detrimental conflicts. You cannot actually reduce how much is overridden through load order alone, but it is often the case that some overrides are acceptable while others cause problems in game. Setting a good load order is the process by which you seek to maximise the former instead of the latter.

Further compatibility between mods is then possible through the use of patches, which are made to address incompatibilities between specific mods, and through the use of Wrye Bash, which can create a *Bashed Patch* that allows you to select between types of changes for different mods. For example, two mods change a piece of armour, but one changes how it looks and the other changes its effectiveness: through load order alone you could only ever get one change, but using Wrye Bash you can get both, because they change different things in the same armour entry.

A plugin's position in the load order is often displayed by mod managers as a hexadecimal number, from 00 to FE (0 to 255 in decimal). The plugin with position 00 loads first, and the plugin with position FE is loaded last. Hexadecimal numbers are used to display the load order positions of plugins because these numbers form the first two digits of the code that the game uses to reference the entries that the plugin adds, so knowing the numbers allows modders and mod users to determine which plugin an entry is from.

So how does the game decide on the load order to use? Well, there are currently two different methods used, depending on the game (and its version):

- In Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas, load order is decided by the relative timestamps of plugins in the game's Data directory. An installed plugin's load order is therefore an intrinsic property of that plugin. The active plugin with the earliest date loads first, with plugins being listed in descending date order (earliest to latest).
- In Skyrim and Fallout 4, load order is decide by the order in which plugins are listed in ``plugins.txt``. This brought with it a fundamental change, in that load order position is no longer an intrinsic property of a plugin.

There are a few hardcoded rules that trump load order, irrespective of the method used to determine it:

- Master plugins always load before non-master plugins.
- In Skyrim and Fallout 4, the game's main master file (``Skyrim.esm`` or ``Fallout4.esm`` respectively) always loads before all other plugins.
- In Skyrim, if ``Update.esm`` is present, it is always loaded. If it has no load order position set, it loads after all other master plugins. That might sound a bit odd, but recall that in Skyrim and Fallout 4 load order position is not an intrinsic property of a plugin.

Active Plugins
==============

We've covered what load order is, how it is used, and how the game determines it, but we haven't covered how the game knows which plugins to load, ie. which plugins are active. Once again, there are two systems used for deciding which plugins are active, depending on the game.

- Morrowind lists the filenames of the plugins that are currently active in its ``Morrowind.ini`` file, found in the game's install folder.
- The other games list the plugins that are currently active in their ``plugins.txt``, by default found in the game's ``%LOCALAPPDATA%`` folder, but it can be relocated to the game's install folder by toggling an option in that game's ini file.

Both active plugins files are encoded in Windows-1252. This is fairly important, as it means that some plugins have filenames which cannot be represented in Windows-1252, and so cannot be activated without first renaming them.

Up to 255 plugins can be active at any one time. Listing more than 255 plugins in the active plugins file will result in weirdness and broken things, as will listing a plugin more than once.

An immediate consequence of Skyrim and Fallout 4 using ``plugins.txt`` to store both load order and active plugin information is that inactive plugins cannot be said to have any load order position. This might not seem a problem at first, since the game only cares about the relative order of the plugins it loads, but modders have engineered methods by which inactive plugins can have their changes loaded by another active plugin (eg. Wrye Bash's Bashed Patch). When any such method is used, the load order of the inactive plugins is required to resolve any dependencies and conflicts they have, both with each other and with any active plugins. As such, it becomes necessary to re-implement a system that assigns a load order position to all installed plugins, hence the textfile-based load order standard.

False-Flagged Plugins
=====================

Whether a mod plugin is a master file or a non-master file is dependent not on its file extension (``.esp``, ``.esm``), but is instead decided by the value of a setting inside the plugin file, known as its *master bit flag*. Master files have this set to ``1``, and non-master files have it set to ``0``. However, because ESM is an acronym for *Elder Scrolls Master*, and ESP is an acronym for *Elder Scrolls Plugin* (even for Fallout 3 and Fallout: New Vegas), most people assume that master plugins are files with ``.esm`` extensions, while non-master plugins are files with ``.esp`` extensions.

This assumption is valid for the most part, as the vast majority of masters are ``.esm`` files, and the vast majority of plugins are ``.esp`` files. However, sometimes this is not the case. When a plugin has a ``.esm`` extension but has a master bit flag value of ``0``, or has an extension of ``.esp`` and a master bit flag value of ``1``, the plugin is said to be *false-flagged*, as its extension does not match its master bit flag value. False-flagged plugins are most common in Fallout 3 due to the use of FO3Edit's *Master Update* feature, which turns non-master plugins into false-flagged plugins, to avoid bugs that only manifest for non-master plugins.

The distinction between masters and non-masters is important for load order because the game will always load all masters before all non-masters, regardless of whether they are listed or dated (depending on your game) as such by the mechanism used to determine load order. Earlier versions of most if not all modding utilities, put all ``.esm`` files before all ``.esp`` files, but failed to take into account the value of the master bit flag of plugins. As such, the load order they displayed was incorrect, as it failed to take into account the game always loading masters before plugins. More recent versions of the most popular modding utilities, at least, properly display all master files before all plugin files.
