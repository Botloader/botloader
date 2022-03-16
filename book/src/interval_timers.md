# Interval timers

Botloader provides [Script.onInterval](/docs/classes/Script.html#onInterval) for executing a function on a interval.

Keep in mind that as mentioned in the [script lifecycle](script_lifecycle.md) section that your server's vm gets shut down whenever it's not in use, so your vm may shut down between these intervals and any variables set will be lost. (see the Storage section for persistent storage)

The function takes in 2 arguments, the name of the timer used for tracking when the interval last ran, and the interval itself.

The interval an be specified in 2 formats:
 - Number of minutes
 - Cron style string

## Cron strings

Using the cron style string it can be useful to use an online tool such as [crontab.guu](https://crontab.guru/) to make the timer.

Here's an example cron string for executing a function every day at 1pm (UTC): `0 13 * * *` 