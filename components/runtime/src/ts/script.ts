// imports for side effects, making them available in scripts always
import "./core_util";


import { Commands } from "./commands";
import { Internal } from "./generated";
import { EventSystem } from "./eventsystem";
import { OpWrappers } from "./op_wrappers";
import { Storage } from "./storage";
import { Tasks } from "./scheduled_tasks";
import { ComponentInteraction, Interaction, SelectMenuInteraction } from "./discord";

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
    private commands: Commands.Command[] = [];

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
        this.commands.push(command);
        EventSystem.commandSystem.addCommand(command);
    }

    /**
     * @deprecated use {@link createStorageJson}
     */
    createGuildStorageJson<T>(namespace: string) {
        let bucket = new Storage.JsonBucket<T>(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
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
     * let funStorage = script.createStorageJson(new Storage.JsonBucket<Data>("fun-data"));
     * ```
     *
     */
    createStorageJson<T>(namespace: string) {
        let bucket = new Storage.JsonBucket<T>(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * @deprecated use {@łink createStorageNumber}
     */
    createGuildStorageNumber(namespace: string) {
        let bucket = new Storage.NumberBucket(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * Creates a new number storage bucket for persistent storage.
     * 
     * This is the same as {@link createStorageJson} except that this bucket can only store number values, the upside of this is that it can be sorted.
     * 
     * An example use case could be storing the scores of users in a leveling system with the key being their user ID and the value being their score
     * this way you can use {@link Storage.NumberBucket.sortedList} to get a sorted list of entries.
     * 
     * See {@link createStorageJson} for more general info on storage buckets
     */
    createStorageNumber(namespace: string) {
        let bucket = new Storage.NumberBucket(namespace);
        this.storageBuckets.push(bucket);

        return bucket;
    }

    /**
     * Create a new persistent variable.
     * 
     * This is built on top of storage buckets so you can think of it as a single entry inside a storage bucket.
     * 
     * @param key Key for the variable inside the storage bucket, the same key in another script will refer to the same entry
     * @param options Additional options
     * @returns 
     */
    createStorageVarJson<T>(key: string, options?: StorageVarExtraOptions) {
        const namespace = options?.namespace ?? "bl:vars_json";
        return new Storage.JsonVar<T>(namespace, key);
    }

    /**
     * Create a new persistent variable.
     * 
     * This is built on top of storage buckets so you can think of it as a single entry inside a storage bucket.
     * 
     * @param key Key for the variable inside the storage bucket, the same key in another script will refer to the same entry
     * @param options Additional options
     * @returns 
     */
    createStorageVarNumber(key: string, options?: StorageVarExtraOptions) {
        const namespace = options?.namespace ?? "bl:vars_number";
        return new Storage.NumberVar(namespace, key);
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

    onInteractionButton<T>(name: string, cb: (interaction: ComponentInteraction, extraData: T) => any) {
        EventSystem.onInteractionButton(name, cb);
    }
    onInteractionSelectMenu<T>(name: string, cb: (interaction: SelectMenuInteraction, extraData: T) => any) {
        EventSystem.onInteractionSelectMenu(name, cb);
    }
    // onInteractionModalSubmit<T>(name: string, cb: (ctx: InteractionContext, submittedValues: SubmittedComponentValue[], data: T) => any) { }

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
    on(eventType: "MESSAGE_REACTION_ADD", cb: (evt: EventSystem.EventTypes["MESSAGE_REACTION_ADD"]) => void): void;
    on(eventType: "MESSAGE_REACTION_REMOVE", cb: (evt: EventSystem.EventTypes["MESSAGE_REACTION_REMOVE"]) => void): void;
    on(eventType: "MESSAGE_REACTION_REMOVE_ALL", cb: (evt: EventSystem.EventTypes["MESSAGE_REACTION_REMOVE_ALL"]) => void): void;
    on(eventType: "MESSAGE_REACTION_REMOVE_ALL_EMOJI", cb: (evt: EventSystem.EventTypes["MESSAGE_REACTION_REMOVE_ALL_EMOJI"]) => void): void;

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

        const [cmds, groups] = this.genCommandsBinding();

        OpWrappers.scriptStarted({
            description: this.description,
            commands: cmds,
            commandGroups: groups,
            scriptId: this.scriptId,
            intervalTimers: this.intervalTimers.map(inner => inner.timer),
            taskNames: this.taskHandlerNames,
        });

        EventSystem.registerEventMuxer(this.events);

        this.events.on("BOTLOADER_INTERVAL_TIMER_FIRED", this.handleIntervalEvent.bind(this));
    }

    private async handleIntervalEvent(evt: Internal.IntervalTimerEvent) {
        const timer = this.intervalTimers.find(timer => timer.timer.name === evt.name);
        if (timer) {
            await timer.callback();
        }
    }

    /**
     * @internal
     */
    genCommandsBinding(): [Internal.Command[], Internal.CommandGroup[]] {

        const commands: Internal.Command[] = this.commands.map(cmd => {
            const options: Internal.CommandOption[] = [];
            for (let prop in cmd.options) {
                if (Object.prototype.hasOwnProperty.call(cmd.options, prop)) {
                    let entry = cmd.options[prop];
                    options.push({
                        name: prop,
                        description: entry.description,
                        kind: entry.kind,
                        required: entry.required || false,
                        extraOptions: entry.extraOptions,
                    })
                }
            }

            let group = undefined;
            let subGroup = undefined;
            if (cmd.group) {
                if (cmd.group.parent) {
                    group = cmd.group.parent.name;
                    subGroup = cmd.group.name;
                } else {
                    group = cmd.group.name;
                }
            }

            return {
                name: cmd.name,
                description: cmd.description,
                options: options,
                kind: cmd.kind,
                group,
                subGroup,
            }
        });

        const groups: Internal.CommandGroup[] = [];

        OUTER:
        for (let cmd of this.commands) {
            if (cmd.group) {
                if (groups.some(g => g.name === cmd.group?.name)) {
                    continue OUTER;
                }

                // new group
                groups.push({
                    name: cmd.group.name,
                    description: cmd.group.description,
                    subGroups: cmd.group.subGroups.map(sg => {
                        return {
                            name: sg.name,
                            description: sg.description
                        }
                    })
                })
            }
        }

        return [commands, groups];
    }
}

interface IntervalTimerListener {
    timer: Internal.IntervalTimer,
    callback: () => any,
}

interface StorageVarExtraOptions {
    namespace?: string,
}