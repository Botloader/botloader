// imports for side effects, making them available in scripts always
import "./core_util";


import { Commands } from "./commands";
import { Ops, Events, Discord } from "./models";
import { InternalEventSystem, EventMuxer, EventTypes } from "./events";
import { OpWrappers } from "./op_wrappers";
import { Storage } from "./storage";
import { HttpClient } from "./httpclient";
import { UpdateGuildMemberFields } from "./models/ops/UpdateGuildMemberFields";
import { Tasks } from "./scheduled_tasks";

/**
 * The script class is the main way you interact with botloader and discord.
 */
export class Script {

    readonly httpClient: HttpClient.Client;

    readonly scriptId: number;
    readonly description: string;

    private events = new EventMuxer();
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
        this.httpClient = new HttpClient.Client(id);
    }

    on(eventType: "MESSAGE_DELETE", cb: (evt: EventTypes["MESSAGE_DELETE"]) => void): void;
    on(eventType: "MESSAGE_UPDATE", cb: (evt: EventTypes["MESSAGE_UPDATE"]) => void): void;
    on(eventType: "MESSAGE_CREATE", cb: (evt: EventTypes["MESSAGE_CREATE"]) => void): void;
    on(eventType: "MEMBER_ADD", cb: (evt: EventTypes["MEMBER_ADD"]) => void): void;
    on(eventType: "MEMBER_UPDATE", cb: (evt: EventTypes["MEMBER_UPDATE"]) => void): void;
    on(eventType: "MEMBER_REMOVE", cb: (evt: EventTypes["MEMBER_REMOVE"]) => void): void;
    on<T extends keyof EventTypes>(eventType: T, cb: (evt: EventTypes[T]) => void): void {
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
    registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>) {
        this.commandSystem.commands.push(cmd as Commands.CommandDef<Commands.OptionsMap>);
    }

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
    registerIntervalTimer(name: string, interval: string | number, callback: () => any) {
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
    registerStorageBucket<T extends Storage.Bucket<U>, U>(bucket: T): T {
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
    async registerTaskHandler<T>(namespace: string, cb: (task: Tasks.Task<T>) => any) {
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
        InternalEventSystem.registerEventMuxer(this.events);

        this.events.on("BOTLOADER_INTERVAL_TIMER_FIRED", this.onInterval.bind(this));
    }

    private async onInterval(evt: Events.IntervalTimerEvent) {
        const timer = this.intervalTimers.find(timer => timer.timer.name === evt.name);
        if (timer) {
            await timer.callback();
        }
    }

    // Guild functions
    getGuild(): Promise<Discord.Guild> {
        return OpWrappers.getGuild()
    }
    // editGuild() { }

    // Message functions
    getMessage(channelId: string, messageId: string): Promise<Discord.Message> {
        return OpWrappers.getMessage({
            channelId,
            messageId,
        })
    }

    getMessages(channelId: string, options?: GetMessagesOptions): Promise<Discord.Message[]> {
        return OpWrappers.getMessages({
            channelId,
            after: options?.after,
            before: options?.before,
            limit: options?.limit,
        })
    }

    createMessage(channelId: string, fields: Ops.OpCreateMessageFields): Promise<Discord.Message> {
        return OpWrappers.createChannelMessage({
            channelId,
            fields,
        });
    }
    editMessage(channelId: string, messageId: string, fields: Ops.OpEditMessageFields): Promise<Discord.Message> {
        return OpWrappers.editChannelMessage({
            channelId,
            messageId,
            fields,
        });
    }

    deleteMessage(channelId: string, messageId: string) {
        return OpWrappers.deleteChannelMessage({
            channelId,
            messageId,
        })
    }

    bulkDeleteMessages(channelId: string, ...messageIds: string[]) {
        return OpWrappers.deleteChannelMessagesBulk({
            channelId,
            messageIds,
        })
    }

    // Role functions
    getRole(roleId: string): Promise<Discord.Role> {
        return OpWrappers.getRole(roleId);
    }
    getRoles(): Promise<Discord.Role[]> {
        return OpWrappers.getRoles();
    }

    // createRole() { }
    // editRole() { }
    // deleteRole() { }

    // Channel functions
    getChannel(channelId: string): Promise<Discord.GuildChannel> {
        return OpWrappers.getChannel(channelId);
    }
    getChannels(): Promise<Discord.GuildChannel[]> {
        return OpWrappers.getChannels();
    }

    // createChannel() { }
    // editChannel() { }
    // deleteChannel() { }

    // Invite functions
    // getInvite() { }
    // getInvites() { }
    // createInvite() { }
    // deleteInvite() { }

    // // Emoji functions
    // getEmoji() { }
    // getEmojis() { }
    // createEmoji() { }
    // editEmoji() { }
    // deleteEmoji() { }


    // // Sticker functions
    // getSticker() { }
    // getStickers() { }
    // createSticker() { }
    // editSticker() { }
    // deleteSticker() { }

    async getMember(id: string): Promise<Discord.Member | undefined> {
        return (await OpWrappers.getMembers([id]))[0] || undefined;
    }

    async getMembers(ids: string[]): Promise<(Discord.Member | null)[]> {
        return await OpWrappers.getMembers(ids);
    }

    async updateMember(userId: string, fields: UpdateGuildMemberFields): Promise<Discord.Member> {
        return await OpWrappers.updateMember(userId, fields);
    }

    async addMemberRole(userId: string, roleId: string): Promise<void> {
        return await OpWrappers.addMemberRole(userId, roleId);
    }

    async removeMemberRole(userId: string, roleId: string): Promise<void> {
        return await OpWrappers.removeMemberRole(userId, roleId);
    }
}

interface IntervalTimerListener {
    timer: Ops.IntervalTimer,
    callback: () => any,
}

export interface GetMessagesOptions {
    /**
     * Limit max results, max 100, default 50
     */
    limit?: number,

    /**
     * Return messages made after this message id
     */
    after?: string,
    /**
     * Return messages made before this message id
     */
    before?: string,
}

