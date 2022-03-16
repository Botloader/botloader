# Timers

Botloader does not provide a `setTimeout` or `setInterval` function that you may be familiar from the browser for reasons being it would keep your vm loaded while doing no work, effectively wasting resources. To get around this botloader provides the following:

 - [Script.onInterval](interval_timers.md) for executing a function on a interval. [API](https://botloader.io/docs/classes/Script.html#onInterval)
 - [Scheduled Tasks](scheduled_tasks.md) for scheduling something to be run at a specific time in the future.