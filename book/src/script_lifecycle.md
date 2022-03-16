# Script lifecycle

Internally botloader uses v8 for running your scripts but keeping all your scripts loaded in a v8 vm forever would be pretty inefficient this is why botloader will shut down your vm when it's not needed and start it again once it's needed.

Botloader will not shut down your vm until it's "done" (done meaning there ae no more pending futures), or you have exceeded your quota (there is no quota at the time of writing, there will be premium plans in the future extending this quota).

But after your vm is shut down all variables set in it is lost. To get around this you can use the storage API, out key value based database to keep persistent data and state in.

When you change a script, botloader will also re-run all your scripts as it needs to know what new timers, task handlers, event handlers and so on have been added/removed.