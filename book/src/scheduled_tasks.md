# Scheduled tasks

[Task namespace API docs](/docs/modules/Tasks.html)

Scheduled tasks provides the ability to run tasks at a specific time and date in the future. An example use case for ths would be a mute command where you would scheduled a un-mute task to un-mute them in the future.

## Task buckets

The task system is based around the task bucket system, you define a namespace and the optional data attached to tasks in this namespace alongside a callback function that runs when tasks in this bucket is due.

Here is an example reminder command that sends a message at a certain time in the future:
```ts
import { Commands, Discord } from "botloader";

interface ReminderData {
    userId: string,
    message: string,
    channelId: string,
}

// This task bucket defines a namespace "reminders" that has tasks with the above
// "ReminderData" in them, used to handle the reminder task.
const reminders = script.createTaskBucket<ReminderData>({
    name: "reminders",
}, async (task) => {
    await Discord.createMessage(task.data.channelId, {
        content: `Reminder for <@${task.data.userId}>: ${task.data.message} `
    })
})

script.createSlashCommand("remindme", "Set a reminder for yourself")
    .addOptionInteger("minutes", "in how many minutes")
    .addOptionString("message", "reminder message")
    .build(async (ctx, args) => {
        const execAt = new Date(Date.now() + (args.minutes * 60 * 1000));

        const data: ReminderData = {
            message: args.message,
            userId: ctx.member.user.id,
            channelId: ctx.channelId,
        }

        await reminders.schedule({
            data: data,
            executeAt: execAt,
        });

        await ctx.createFollowup(`Reminder scheduled for <t:${Math.floor(execAt.getTime() / 1000)}> `)
    });
```

## Optional unique "Key"

Tasks can also optionally have a unique key, this `key` can be used to ensure there is no duplicate tasks for this `key`, and also used to overwrite previous tasks scheduled using the same `key`. A use case for this would be a mute command where if you want to change the mute duration of a already muted user you can set the `key` to the user's Id, this way when you call `Tasks.schedule` you will overwrite the previous scheduled task for un-muting this user with the new date and time.
