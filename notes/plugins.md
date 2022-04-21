**blcmd**

Command line option replacing most of the functionality in the vscode plugin.

For now, only needs the plugin specific stuff.

Sub sections:

- http api client
- config loading/saving from logging in

**new database tables**

- Plugins table
- Versions table
  - A version of the plugin, along with all the plugin data in a tarball(?)
  - Probably filter out only the needed contents for the tarball, ts files and
    the botloader-plugin.json file
  - 3 version types, stable, pre-release, development (rapidly changing, only
    last one kept)
- GuildPlugins table
  - Holds the plugins the guild has added to it along with their version

**support in vmworker and scheduler**

- GuildHandler, maybe split up worker sessions or vm sessions into their own
  thing?
