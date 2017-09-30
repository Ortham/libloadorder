## Active Plugins

We've covered what load order is, how it is used, and how the game determines it, but we haven't covered how the game knows which plugins to load, ie. which plugins are active. Once again, there are two systems used for deciding which plugins are active, depending on the game.

- Morrowind lists the filenames of the plugins that are currently active in its `Morrowind.ini` file, found in the game's install folder.
- The other games list the plugins that are currently active in their `plugins.txt`, by default found in the game's `%LOCALAPPDATA%` folder, but it can be relocated to the game's install folder by toggling an option in that game's ini file.

Both active plugins files are encoded in Windows-1252. This is fairly important, as it means that some plugins have filenames which cannot be represented in Windows-1252, and so cannot be activated without first renaming them.

Up to 255 plugins can be active at any one time. Listing more than 255 plugins in the active plugins file will result in weirdness and broken things, as will listing a plugin more than once.

An immediate consequence of Skyrim and Fallout 4 using `plugins.txt` to store both load order and active plugin information is that inactive plugins cannot be said to have any load order position. This might not seem a problem at first, since the game only cares about the relative order of the plugins it loads, but modders have engineered methods by which inactive plugins can have their changes loaded by another active plugin (eg. Wrye Bash's Bashed Patch). When any such method is used, the load order of the inactive plugins is required to resolve any dependencies and conflicts they have, both with each other and with any active plugins. As such, it becomes necessary to re-implement a system that assigns a load order position to all installed plugins, hence the textfile-based load order standard.
