This is a pretty simple crate for handling log entries that should be sent to guilds, for example script errors.

Its made like this:
 1. Make a GuildLoggerBuilder
 2. Add backends to it
 3. Call run() on it and you can use the return of that to log stuff to guilds.

Backends are what handles the actual log entries the current ones are:
 - DiscordLogger: sends log entries to the guilds configured logging channel
 - GuildSubscriberBackend: Handles dynamic subscriptions to a guilds logs, such as to the websocket (over botrpc)