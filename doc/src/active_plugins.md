## Active Plugins

We've covered what load order is, how it is used, and how the game determines it, but we haven't covered how the game knows which plugins to load, ie. which plugins are active. Once again, there are two systems used for deciding which plugins are active, depending on the game.

- Morrowind lists the filenames of the plugins that are currently active in its `Morrowind.ini` file, found in the game's install folder.
- The other games list the plugins that are currently active in their `plugins.txt`, by default found in the game's `%LOCALAPPDATA%` folder, but it can be relocated to the game's install folder by toggling an option in that game's ini file.

Both active plugins files are encoded in Windows-1252. This is fairly important, as it means that some plugins have filenames which cannot be represented in Windows-1252, and so cannot be activated without first renaming them.

Up to 255 plugins can be active at any one time. Listing more than 255 plugins in the active plugins file will result in weirdness and broken things, as will listing a plugin more than once.

In Skyrim Special Edition and Fallout 4, light masters do not count towards the 255 active plugin limit, and instead have a separate limit of up to 4096 active light masters.
