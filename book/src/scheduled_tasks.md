# Scheduled tasks

[Task namespace API docs](/docs/modules/Tasks.html)

Scheduled tasks provides the ability to run tasks at a specific time and date in the future. An example use case for ths would be a mute command where you would scheduled a un-mute task to un-mute them in the future.

To use scheduled tasks you need to register a handler for the task `name` through [script.onTask](/docs/classes/Script.html#onTask).

After you have registered a handler you can call [Tasks.schedule](/docs/modules/Tasks.html#schedule) to schedule a new task.

When scheduling tasks you have the option of attaching some data to it, this data will be passed on to the handler when the task is being ran.

Tasks can also optionally have a unique key, this `key` can be used to ensure there is no duplicate tasks for this `key`, and also used to overwrite previous tasks scheduled using the same `key`. A use case for this would be a mute command where if you want to change the mute duration of a already muted user you can set the `key` to the user's Id, this way when you call `Tasks.schedule` you will overwrite the previous scheduled task for un-muting this user with the new date and time.