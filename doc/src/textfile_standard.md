# The Textfile-Based Load Order Standard

The textfile-based load order standard was developed by Lojack (Wrye Bash), WrinklyNinja (BOSS), Kaburke (Nexus Mod Manager) and Dark0ne (owner of the Nexus sites) to serve the community's need for the ability to assign load order positions to all installed plugins in Skyrim (not the Special Edition).

The standard dictates that:

- An installed plugin is defined to be a plugin that is located at the location which the game scans to detect plugins.
- Total load order is defined as a load order which assigns a position to every installed plugin at the time at which the load order is recorded.
- A blank line is defined as a string that contains only a CRLF line break.
- A comment line is defined as a string that begins with a hash character (`#`) and ends with a CRLF line break.
- A plugin line is defined as a string that begins with a plugin filename (basename.extension, where the extension is `esp` or `esm`), which is followed immediately by a CRLF line break.
- Total load order is stored in a `loadorder.txt` file.
- `loadorder.txt` is encoded in UTF-8, *without* a Byte Order Mark.
- `loadorder.txt` is stored alongside the `plugins.txt`, in whichever location that is.
- `loadorder.txt` contains only blank lines, comment lines and plugin lines.
- `loadorder.txt` lists the game's main master file first.
- `loadorder.txt` lists all master files before all plugin files.
- `loadorder.txt` contains no duplicate plugin lines.
- `plugins.txt` is encoded in Windows-1252.
- `plugins.txt` contains only blank lines, comment lines and plugin lines.
- `plugins.txt` contains no more than 255 plugins.
- `plugins.txt` contains no duplicate plugin lines.
- The order of the plugins listed in `plugins.txt` is identical to their order in `loadorder.txt`.

There are some situations identified that might arise for which there is no required behaviour defined. Nevertheless, the standard makes some recommendations for the handling of these situations, detailed below.

An attempt is made to activate a plugin with a filename that cannot be represented in the Windows-1252 encoding.
  Utilities check that plugin filenames can be represented correctly in Windows-1252 before attempting to activate them. Plugins with filenames that cannot be represented correctly should be skipped, and the user/client made aware of the issue.

Since `loadorder.txt` was last written, plugins have been installed or uninstalled.
  Plugins that have been uninstalled should have their lines in loadorder.txt removed, and plugins that have been installed should have lines appended to loadorder.txt.

Since `plugins.txt` was last written, active plugins have been uninstalled.
  Active plugins that have been uninstalled should have their lines in `plugins.txt` removed.

The order of the plugins listed in `plugins.txt` is not identical to their order in `loadorder.txt`.
  Utilities check for synchronisation on startup and maintain it throughout their operation, rather than re-synchronising the files on program close, for instance. This is to prevent issues for any other programs open at the same time. If desynchronisation is detected, the only standard-based recovery option is to derive *plugins.txt* from *loadorder.txt*, first getting a list of filenames to be written from *plugins.txt*. Alternatively, a utility could use some internal load order backup to restore from. See the coloured box below for a more detailed breakdown of the issue.

## The Desynchronisation Problem

If either `plugins.txt` or `loadorder.txt` are changed such that the load order of the plugins in `plugins.txt` is not the same in `plugins.txt` and `loadorder.txt`, then the difference cannot generally be precisely resolved without discarding one file's ordering. This is due to the load order of plugins in plugins.txt being weakly defined, ie. it is defined relative to other active plugins, but not in relation to inactive plugins. An example:

If you use a standard-compliant tool to set a load order of A,b,c,d,E,f,g where uppercase letters denote active plugins, then you use the Skyrim launcher to move A after E, then `plugins.txt` will read E,A while `loadorder.txt` remains unchanged. There is no way of knowing from the contents of plugins.txt whether you moved A after E or E before A. If these were the only two plugins, then it would not be an issue, but you also have inactive plugins interspersed amongst them, so you have the following possibilities, all of which are potentially valid, but also potentially damaging in terms of conflicts, etc.:

- b,c,d,E,A,f,g
- b,c,d,E,f,A,g
- b,c,d,E,f,g,A
- E,A,b,c,d,f,g

There is no way of knowing which is the correct order to choose for the full load order based on the active load order. You must therefore choose to use one of the two files' orderings. Since `plugins.txt` does not define the load order positions of inactive plugins, it is unsuitable, and `loadorder.txt` must be used. The alternative would be for a utility to restore load order from their own internal backup, hence why the standard does not define a specific behaviour, as it may be `loadorder.txt` that was altered and is now wrong.
