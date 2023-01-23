# Script development through vs code

## Setting up the extension

To use the visual studio code extension, download the `botloader` extension from the marketplace.

After that you need to grab an API key from the botloader.io website, you do this by logging in and navigating to your [user settings](/settings).

Once you have copied a API key you can press `ctrl-shift-p` in vs code to bring up the command palette then use the `botloader set api key` command

## Set up a workspace

To manage scripts on a server you must setup a workspace. To set up a workspace open the command palette and issue the `botloader set up workspace` command, this will bring up a guild selection afterwards followed by picking a persistent or temporary directory to set up the workspace in.

Once you're in the workspace you will see a `.botloader` folder that holds the internal state of the workspace, this is how the extension is able to work so don't touch this folder as you might mess it up.

## Creating, editing and removing scripts

All your scripts are located at the top level folder in the workspace (same level as the `.botloader` folder).

To create a script imply create a new file with the `.ts` suffix in the top-level folder, for example `hello_world.ts`.

You can edit all scripts in the folder as they were normal files, but changes won't be sent until you push them, see the next section for that.

To delete a script simply delete the file for it.

## Deploying your changes to botloader

Any changes you make in the workspace will not appear until you deploy or push them to botloader, this is done by navigating to the "scm" or "source control" tab in vs code on the left side. In there you should see a list of added, changed and removed scripts and at the top and next to each file you should have the option to push the changes. Once you push the changes botloader will run your scripts.