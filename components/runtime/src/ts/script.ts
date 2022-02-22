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

    on(eventType: "MESSAGE_DELETE", cb: (evt: EventSystem.EventTypes["MESSAGE_DELETE"]) => void): void;
    on(eventType: "MESSAGE_UPDATE", cb: (evt: EventSystem.EventTypes["MESSAGE_UPDATE"]) => void): void;
    on(eventType: "MESSAGE_CREATE", cb: (evt: EventSystem.EventTypes["MESSAGE_CREATE"]) => void): void;
    on(eventType: "MEMBER_ADD", cb: (evt: EventSystem.EventTypes["MEMBER_ADD"]) => void): void;
    on(eventType: "MEMBER_UPDATE", cb: (evt: EventSystem.EventTypes["MEMBER_UPDATE"]) => void): void;
    on(eventType: "MEMBER_REMOVE", cb: (evt: EventSystem.EventTypes["MEMBER_REMOVE"]) => void): void;
    on<T extends keyof EventSystem.EventTypes>(eventType: T, cb: (evt: EventSystem.EventTypes[T]) => void): void {
        this.events.on(eventType, cb);
    }

    /**
     * Register a command to this guild.
     * 
     * @param cmd The command to register
     * 
     * @example ```ts
     * * script.registerCommand({
     *     name: "sub",
     *     description: "subtracts 2 numbers",
     *     group: mathGroup,
     *     options: {
     *         "a": { description: "a", kind: "Integer", required: true },
     *         "b": { description: "b", kind: "Integer", required: true },
     *     },
     *     callback: async (ctx, args) => {
     *         const result = args.a - args.b;
     *         await ctx.sendResponse(`Result: ${result}`)
     *     }
     * });
     * ```
     */
    // registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>) {
    //     this.commandSystem.commands.push(cmd as Commands.CommandDef<Commands.OptionsMap>);
    // }

    addCommand(command: Commands.Command) {
        this.commandSystem.commands.push(command);
    }
    // addSlashCommand() { },
    // addUserCommand() { },
    // addMessageCommand() { },

    /**
     * 
     * @param name The name of the timer
     * @param interval The interval, either in minutes for running the callback at every x minutes, or a cron style timer. 
     * 
     * https://crontab.guru/ is a neat helper for making cron intervals 
     * 
     * @param callback Callback to run at every interval
     * 
     * @example ```ts
     *  script.registerIntervalTimer("gaming", "*\/5 * * * *", () => {
     *     // do stuff here
     * });
     * ```
     */
    addIntervalTimer(name: string, interval: string | number, callback: () => any) {
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

    /**
     * Register a storage bucket to the script
     * 
     * Note that the same storage bucket can be registered in multiple scripts, and you can use this to share data betweem scripts.
     *
     * @param bucket The bucket itself
     * @returns The registered bucket
     * 
     * @example ```ts
     * interface Data{
     *     key: string,
     * }
     * script.registerStorageBucket(new Storage.JsonBucket<Data>("fun-data"));
     * ```
     */
    // addStorageBucket<T extends Storage.Bucket<U>, U>(bucket: T): T {
    //     this.storageBuckets.push(bucket);
    //     return bucket;
    // }

    addStorageBucketJson<T>(namespace: string) {
        let bucket = new Storage.JsonBucket<T>(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    addStorageBucketNumber(namespace: string) {
        let bucket = new Storage.NumberBucket(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * Register a scheduled task handler for the provided namespace
     * 
     * See {@link Tasks} for more info about scheduled tasks
     * 
     * @param namespace The task namespace to handle tasks from
     * @param cb The callback function to run
     */
    async addTaskHandler<T>(namespace: string, cb: (task: Tasks.Task<T>) => any) {
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

        this.events.on("BOTLOADER_INTERVAL_TIMER_FIRED", this.onInterval.bind(this));
    }

    private async onInterval(evt: Internal.IntervalTimerEvent) {
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
