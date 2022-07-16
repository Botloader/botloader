# Echo command tutorial

This is a short tutorial on how to make a simple `echo` command that echoes back the input

## Setup

First step is to create a script, if you're using the web editor go to your server settings -> scripts in the sidebar, and create a new script with the name `echo`. (Or whatever else you want to call it.) and now you should be editing this script! Note that this script is not yet enabled, there's a button to enable it in the sidebar.

## Creating the command

First, we want to import the `Commands` namespace, all botloader API's are exported under `botloader` so you can import the commands namespace like so:

```ts
import { Commands } from 'botloader';
```

Then it's creating the command itself, all scripts in botloader have a `script` variable that you use to set up various things such as event listeners, storage buckets and so on, including commands.

So, through the `script` we create a new command:

```ts
script.createCommand(/* TODO: what do we pass it? */);
```

This createCommand function takes in a command object that can be created from either the slash command, user command or message command builders, in this tutorial we'll be using the slash command builder. The user and message commands show up when you right click on users and messages, while slash commands shows up when you type a `/` in the chat input area on discord.

`Commands.slashCommand` takes a name and description. Note that the name requirements are quite strict, it can't contain spaces or special characters besides `-`

```ts
script.createCommand(Commands.slashCommand("echo", "echoes back your input"))
```

Were not done yet! This command takes in some input from the user to echo back, we need to define this input, and this is called `options` when dealing with commands. To add a `option` use one of the `addOption` methods, were going to use `addOptionString` and this takes in a name and a description, as well as some additional optional options (such as making this option optional/required) that were not gonna go into here.

This is how it looks then:

```ts
script.createCommand(
    Commands.slashCommand("echo", "echo back what i give you")
    .addOptionString("what", "what to echo back")
)
```

I've gone ahead and formatted it a bit nicer to make it more readable.

Were still not done yet! This will still show an error and that's because we must build our command and give it a function to run when someone uses the command, you do that using the `build` method:

```ts
script.createCommand(
    Commands.slashCommand("echo", "echo back what i give you")
    .addOptionString("what", "what to echo back")
    .build(async (ctx, args) => {
        // Code that runs when someone uses the command
    })
)
```

This is now a fully valid command and there should be no errors, if you save it and enable the script in the sidebar, after a short delay (around 10 seconds - a minute), it should show up in discord. But if you run it, it will have no output, it will just show the bot thinking forever...

So the next step is to make the bot echo back what the user gave it. To do that we use the first argument, `ctx` in above example.

This object is a instance of a interaction and the full docs for it can be viewed [here](/docs/classes/Commands.ExecutedCommandContext.html), but the method were going to use to send a response is `ctx.createFollowup` which takes in either a simple string as a message or a more complex object for sending embeds and more.

Then to get the user input we use the second argument given to the callback, in the above example that would be the `args` object. This object has the "parsed" version of the options that we defined for the command, for this command that object looks like this:

```ts
{what: "whatever the user typed in"}
```

So, using those 2 pieces we can send back the user input as a response.

In the end the full script should look like this:

```ts
import { Commands } from 'botloader';

script.createCommand(
    Commands.slashCommand("echo", "echo back what i give you")
    .addOptionString("what", "what to echo back")
    .build(async (ctx, args) => {
        await ctx.createFollowup(args.what);
    })
)
```

The reason we `await` it is so that the function does not return before we send the response, although it's not really needed.