## Plugin Types

Depending on the game, there are up to four types of plugins:

- Master plugins
- Non-master plugins
- Light master plugins
- Light non-master plugins

Light plugins only exist for Skyrim Special Edition, Skyrim VR, Fallout 4 and Fallout 4 VR. These plugin types are related to the three plugin file extensions: `.esm` (*Elder Scrolls Master*), `.esp` (*Elder Scrolls Plugin*) and `.esl` (*Elder Scrolls Light*), but a plugin's file extension doesn't necessarily match its type, which is decided differently for each game.

### Morrowind

Plugins with the `.esm` file extension are master plugins, and plugins with the `.esp` file extension are non-master plugins.

### Oblivion, Skyrim, Fallout 3, Fallout: New Vegas

Plugin files have an internal setting known as their *master flag*. If this is set, the plugin is a master plugin. If it is not set, the plugin is a non-master plugin. File extension is ignored. If the file extension doesn't match the master flag value, the plugin is said to be *false-flagged*. False-flagged plugins are most common in Fallout 3 due to the use of FO3Edit's *Master Update* feature, which turns non-master plugins into false-flagged plugins, to avoid bugs that only manifest for non-master plugins.

### Skyrim Special Edition, Skyrim VR, Fallout 4, Fallout 4 VR

Plugin files have an internal *light plugin flag* setting as well as their master flag setting, and these are used together with the file extension to determine a plugin's type.

- A plugin is a master plugin if its file extension is `.esm`, or if it has its master flag set.
- A plugin is a light master plugin if its file extension is `.esl`, or if it has its master and light plugin flags set.
- A plugin is a light non-master plugin if its file extension is `.esp` and it has its light plugin flag set.
- A plugin is a non-master plugin if its file extension is `.esp` and neither its master flag nor its light plugin flag set.

It's not possible to have a false-flagged plugin, except that a `.esp` plugin may or may not be light.
