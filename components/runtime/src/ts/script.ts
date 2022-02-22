// imports for side effects, making them available in scripts always
import "./core_util";


import { Commands } from "./commands";
import { Internal } from "./generated";
import { EventSystem } from "./eventsystem";
import { OpWrappers } from "./op_wrappers";
import { Storage } from "./storage";
import { Tasks } from "./scheduled_tasks";

/**
 * The script class is the main way you interact with botloader and discord.
 */
export class Script {
    readonly scriptId: number;
    readonly description: string;

    private events = new EventSystem.Muxer();
    private commandSystem = new Commands.System();
    private intervalTimers: IntervalTimerListener[] = [];
    private storageBuckets: Storage.Bucket<unknown>[] = [];
    private taskHandlerNames: string[] = [];

    private runCalled = false;

    /**
     * @internal
     */
    constructor(id: number) {
        this.description = `script id ${id}`;
        this.scriptId = id;
    }

    /**
     * Creates or updates a command 
     * 
     * See {@link Commands.slashCommand}, {@link Commands.messageCommand} and {@link Commands.userCommand}
     * for more info on defining the commands themselves
     */
    createCommand(command: Commands.Command) {
        this.commandSystem.commands.push(command);
    }

    /**
     * Create new json storage buckets for persistent storage
     * 
     * @param namespace a "name" or "id" for the bucket. 
     * This is not script scoped and the same storage bucket can be registered in multiple scripts to have access to the same data, this is perfectly safe.
     * 
     * @example ```ts
     * interface Data{
     *     key: string,
     * }
     * let funStorage = script.createStorageBucketJson(new Storage.JsonBucket<Data>("fun-data"));
     * ```
     *
     */
    createStorageBucketJson<T>(namespace: string) {
        let bucket = new Storage.JsonBucket<T>(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * Creates a new number storage bucket for persistent storage.
     * 
     * This is the same as {@link createStorageBucketJson} except that this bucket can only store number values, the upside of this is that it can be sorted.
     * 
     * An example use case could be storing the scores of users in a leveling system with the key being their user ID and the value being their score
     * this way you can use {@link Storage.NumberBucket.sortedList} to get a sorted list of entries.
     * 
     * See {@link createStorageBucketJson} for more general info on storage buckets
     */
    createStorageBucketNumber(namespace: string) {
        let bucket = new Storage.NumberBucket(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * Register a scheduled task handler for the provided namespace.
     * 
     * See {@link Tasks} for more info about scheduled tasks and how to schedule a new one.
     * 
     * @param namespace The task namespace to handle tasks from
     * @param cb The callback function to run, with the type of the data passed to the task when scheduled
     * 
     * @example ```ts
     * interface Data{
     *     key: string,
     * }
     * script.onTask<Data>("fun-tasks", (task) => {
     *      console.log("hello world");
     * });
     * ```
     */
    async onTask<T>(namespace: string, cb: (task: Tasks.Task<T>) => any) {
        this.taskHandlerNames.push(namespace);

        this.events.on("BOTLOADER_SCHEDULED_TASK_FIRED", async (evt) => {
            if (evt.namespace === namespace) {
                await cb({
                    ...evt,
                    data: evt.data as T,
                });
            }
        })
    }

    /**
     * Creates or resumes a interval timer.
     * 
     * @param name The name of the timer, this is not namespaced to the current script. You could overwrite a timer from another script with the same name.
     * @param interval The interval, either in minutes for running the callback at every x minutes, or a cron style timer. 
     * 
     * https://crontab.guru/ is a neat helper for making cron intervals 
     * 
     * @param callback Callback to run at every interval
     * 
     * @example ```ts
     *  script.onInterval("gaming", "*\/5 * * * *", () => {
     *     // do stuff here
     * });
     * ```
     */
    onInterval(name: string, interval: string | number, callback: () => any) {
        let timerType;
        if (typeof interval === "number") {
            timerType = { minutes: interval };
        } else {
            timerType = { cron: interval };
        }

        this.intervalTimers.push({
            callback,
            timer: {
                name: name,
                interval: timerType,
            }
        });
    }

    on(eventType: "MESSAGE_DELETE", cb: (evt: EventSystem.EventTypes["MESSAGE_DELETE"]) => void): void;
    on(eventType: "MESSAGE_UPDATE", cb: (evt: EventSystem.EventTypes["MESSAGE_UPDATE"]) => void): void;
    on(eventType: "MESSAGE_CREATE", cb: (evt: EventSystem.EventTypes["MESSAGE_CREATE"]) => void): void;
    on(eventType: "MEMBER_ADD", cb: (evt: EventSystem.EventTypes["MEMBER_ADD"]) => void): void;
    on(eventType: "MEMBER_UPDATE", cb: (evt: EventSystem.EventTypes["MEMBER_UPDATE"]) => void): void;
    on(eventType: "MEMBER_REMOVE", cb: (evt: EventSystem.EventTypes["MEMBER_REMOVE"]) => void): void;

    /**
     * Register a general event handler such as for arbitrary discord events like when a new message is sent in the server (MESSAGE_CREATE)
     */
    on<T extends keyof EventSystem.EventTypes>(eventType: T, cb: (evt: EventSystem.EventTypes[T]) => void): void {
        this.events.on(eventType, cb);
    }


    /**
     * @internal
     */
    run() {
        if (this.runCalled) {
            throw new Error("run already called");
        }

        this.runCalled = true;

        const [cmds, groups] = this.commandSystem.genOpBinding();

        OpWrappers.scriptStarted({
            description: this.description,
            commands: cmds,
            commandGroups: groups,
            scriptId: this.scriptId,
            intervalTimers: this.intervalTimers.map(inner => inner.timer),
            taskNames: this.taskHandlerNames,
        });

        this.commandSystem.addEventListeners(this.events);
        EventSystem.registerEventMuxer(this.events);

        this.events.on("BOTLOADER_INTERVAL_TIMER_FIRED", this.handleIntervalEvent.bind(this));
    }

    private async handleIntervalEvent(evt: Internal.IntervalTimerEvent) {
        const timer = this.intervalTimers.find(timer => timer.timer.name === evt.name);
        if (timer) {
            await timer.callback();
        }
    }
}

interface IntervalTimerListener {
    timer: Internal.IntervalTimer,
    callback: () => any,
}
