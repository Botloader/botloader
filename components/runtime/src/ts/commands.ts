import { DiscordModels, Internal } from "./generated";
import { Interaction } from "./discord";

/**
 * The commands namespace provides a command system that works with discord slash commands, as well as 
 * message and user commands (context menu commands).
 * 
 * See the 3 builders: {@link Commands.slashCommand}, {@link Commands.userCommand} and {@link Commands.messageCommand}.
 */
export namespace Commands {

    /**
     * @internal
     */
    export class System {

        commands: Command[] = [];

        addCommand(cmd: Command) {
            if (this.commands.find(v => matchesCommand(cmd, v.name, v.group?.name, v.group?.parent?.name))) {
                throw new Error(`Duplicate commands registered! Cmd: ${cmd.name}, parent: ${cmd.group?.name}, parent of parent: ${cmd.group?.parent?.name}`)
            }

            this.commands.push(cmd);
        }

        /**
         * @internal
         */
        async handleInteractionCreate(interaction: Internal.CommandInteraction) {
            let command = this.commands.find(cmd => matchesCommand(cmd, interaction.name, interaction.parentName, interaction.parentParentName));
            if (!command) {
                return;
            }

            let ctx = new ExecutedCommandContext(interaction);
            if (command.ackMode === "DeferredMessage") {
                await ctx.ackWithDeferredMessage();
            }

            if (interaction.kind === "Chat") {
                let optionsMap: Record<string, any> = {};
                for (let opt of interaction.options) {
                    optionsMap[opt.name] = this.resolveOption(interaction.dataMap, opt.value);
                }

                await command.cb(ctx, optionsMap)
            } else if (interaction.kind === "Message") {
                if (interaction.targetId) {
                    let message = interaction.dataMap.messages[interaction.targetId];
                    await command.cb(ctx, message)
                } else {
                    throw new Error("message not found in datamap")
                }
            } else if (interaction.kind === "User") {
                if (interaction.targetId) {
                    let member = interaction.dataMap.members[interaction.targetId];
                    let user = interaction.dataMap.users[interaction.targetId]
                    let args: InteractionUser = {
                        user,
                        member,
                    };
                    await command.cb(ctx, args)
                } else {
                    throw new Error("member not found in datamap")
                }
            } else {
                throw new Error("Unknown command type")
            }
        }

        private resolveOption(map: Internal.CommandInteractionDataMap, opt: Internal.CommandInteractionOptionValue): unknown {
            switch (opt.kind) {
                case "user":
                    const user = map.users[opt.value];
                    if (user === undefined) {
                        throw new Error("interaction user not found in data map");
                    }

                    const ret: InteractionUser = {
                        user,
                        member: map.members[opt.value],
                    }
                    return ret

                case "role":
                    const role: DiscordModels.Role = map.roles[opt.value];
                    if (role === undefined) {
                        throw new Error("interaction role not found in data map");
                    }

                    return role;

                case "mentionable":
                    let metionableRet: InteractionMentionable;

                    const mentionableUser = map.users[opt.value];
                    if (mentionableUser) {
                        metionableRet = {
                            kind: "User",
                            value: {
                                user: mentionableUser,
                                member: map.members[opt.value],
                            }
                        }
                    } else {
                        let mentionableRole = map.roles[opt.value];
                        if (!mentionableRole) {
                            throw new Error("interaction mentionable (role or user) not found in data map")
                        }

                        metionableRet = {
                            kind: "Role",
                            value: mentionableRole
                        }
                    }

                    return metionableRet;

                case "channel":
                    const channel: Internal.InteractionPartialChannel = map.channels[opt.value];
                    if (channel === undefined) {
                        throw new Error("interaction channel not found in data map");
                    }
                    return channel;

                default:
                    return opt.value;
            }
        }
    }

    function matchesCommand(cmd: Command, name: string, parentName?: string | null, parentParentName?: string | null) {
        if (parentParentName) {
            if (cmd.group && cmd.group.parent) {
                return cmd.name === name && cmd.group.name === parentName && cmd.group.parent.name === parentParentName;
            }
        } else if (parentName) {
            if (cmd.group && !cmd.group.parent) {
                return cmd.name === name && cmd.group.name === parentName;
            }
        } else {
            if (!cmd.group) {
                return cmd.name === name;
            }
        }

        return false
    }

    /**
     * Context information about a command being run
     */
    export class ExecutedCommandContext extends Interaction {
        channelId: string;

        /**
         * Name of the command triggered
         * 
         * Thie field is UNSTABLE and might change later
         * 
         * @internal
         */
        commandName: string;
        /**
         * Parent group of the command that triggered
         * 
         * Thie field is UNSTABLE and might change later
         * 
         * @internal
         */
        parentName?: string;
        /**
         * Parent group of the parent group of the command that triggered 
         * 
         * Thie field is UNSTABLE and might change later
         * 
         * @internal
         */
        parentParentName?: string;

        constructor(interaction: Internal.CommandInteraction) {
            super(interaction.id, interaction.token, interaction.member);

            this.channelId = interaction.channelId;
            this.commandName = interaction.name;
            this.parentName = interaction.parentName ?? undefined;
            this.parentParentName = interaction.parentParentName ?? undefined;
        }
    }


    export interface InteractionUser {
        user: DiscordModels.User,
        member?: Internal.InteractionPartialMember,
    }

    export type InteractionMentionable = {
        kind: "Role",
        value: DiscordModels.Role
    } | {
        kind: "User",
        value: InteractionUser
    }

    /**
     * Raw form of a command handled by botloader
     * 
     * You shouldn't use this directly and instead use one of the builders
     * 
     * This should be considered UNSTABLE and might change in the future
     */
    export interface Command {
        name: string;
        description: string;
        kind: Internal.CommandType,
        group?: Group,
        options?: OptionMap;
        ackMode: AckMode,
        cb: (ctx: {}, args: {}) => any,
    }

    export type OptionType = Option["kind"];

    export type AutocompleteProvider<T> = (data: {}) => Promise<OptionChoice<T>[]> | OptionChoice<T>[];

    export type OptionMap = {
        [key: string]: Option,
    }

    export interface Option {
        kind: "String" | "Number" | "Integer" | "Boolean" | "User" | "Channel" | "Role" | "Mentionable",
        description: string,
        required: boolean,
        extraOptions: StringOption | NumberOption | BooleanOption | ChannelOption | RoleOption | MentionableOption,
    }

    export interface StringOption {
        // choices?: OptionChoice<string>[],
        // autocomplete?: AutocompleteProvider<string>,
    }

    export interface NumberOption {
        // choices?: OptionChoice<number>[],
        minValue?: number,
        maxValue?: number,

        // // autocomplete?: AutocompleteProvider<number>,
    }

    export interface IntegerOption {
        // choices?: OptionChoice<number>[],
        minValue?: number,
        maxValue?: number,

        // // autocomplete?: AutocompleteProvider<number>,
    }

    export interface BooleanOption {

    }

    export interface UserOption {

    }

    export interface ChannelOption {
        channelTypes?: DiscordModels.ChannelType[],
    }

    export interface RoleOption {

    }

    export interface MentionableOption {

    }

    export interface OptionChoice<T> {
        name: string,
        value: T,
    }

    /**
     * A Command group, can only be used with slash commands currently
     * 
     * Groups can have subgroups, but not anything deeper than those 2 levels. This is because of discord restrictions.
     */
    export class Group {
        name: string;
        description: string;
        parent?: Group;
        protected isSubGroup: boolean = false;

        subGroups: Group[] = [];

        /**
         * @param name name of the group as it shows in discord, 1-32 characters (no symbols except - and _)
         * @param description description of the group, 1-100 characters
         */
        constructor(name: string, description: string) {
            this.name = name;
            this.description = description;
        }

        /**
         * Create a subgroup from this group.
         * 
         * Note: subgroups cannot be made from other subgroups.
         * @param name name of the subgroup, 1-32 characters
         * @param description description of the subgroup, 1-100 characters
         * @returns 
         */
        subGroup(name: string, description: string) {
            if (this.isSubGroup) {
                throw "cant make sub groups of sub groups";
            }

            let group = new Group(name, description);
            group.isSubGroup = true;
            this.subGroups.push(group);
            return group;
        }
    }

    /**
     * Create a new slash command builder
     * @param name name of the command, 1-32 characters, (no symbols except - and _)
     * @param description 1-100 character description
     * @returns a builder
     * 
     * 
     * @example ```ts
     * script.createCommand(Commands.slashCommand("echo", "echo's your input")
     * .addOptionString("what", "what to echo")
     * .build(async (ctx, args) => {
     *      await ctx.sendResponse(args.what);
     * }))
     * ```
     */
    export function slashCommand(name: string, description: string) {
        return new SlashCommandBuilder<{}>(name, description, {});
    }

    export type AckMode = "DeferredMessage" | "Custom";

    export class SlashCommandBuilder<TOpts> {
        private name: string;
        private description: string;
        private options: OptionMap;
        private group?: Group;
        private ackMode: AckMode = "DeferredMessage";

        constructor(name: string, description: string, options: OptionMap, group?: Group) {
            this.name = name;
            this.description = description;
            this.options = options;
            this.group = group;
        }

        /**
         * Assigns a group to this command
         * 
         * @example ```ts
         * const group = new Commands.Group("some-group", "some description")
         * script.addCommand(Commands.slashCommand("some-cmd", "some description").setGroup(grou).build(...))
         * ```
         */
        setGroup(group: Group) {
            this.group = group;
            return this;
        }

        /**
         * Set the ack mode of this command
         * (if you're experienced with the discord api, this is the callback type to the interaction)
         * 
         * `DeferredMessage`: It will respond with a public deferred message.
         * `Custom`: You handle the ack'ing of the interaction yourself, this allows you to
         * use ephemeral responses and modals.
         * 
         * 
         * Keep in mind when using custom, you have to ack the interaction within 3 seconds otherwise it will fail.
         */
        setAckMode(mode: AckMode) {
            this.ackMode = mode;
            return this;
        }

        /**
         * See {@link addOption}
         */
        addOptionNumber<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: NumberOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Number", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionString<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: StringOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "String", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionInteger<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: IntegerOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Integer", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionBoolean<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: BooleanOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Boolean", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionUser<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: UserOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "User", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionChannel<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: ChannelOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Channel", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionRole<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: RoleOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Role", description, opts)
        }

        /**
         * See {@link addOption}
         */
        addOptionMentionable<TKey extends string, TRequired extends boolean | undefined>
            (name: TKey, description: string, opts?: MentionableOption & BaseOptionSettings<TRequired>) {
            return this.addOption(name, "Mentionable", description, opts)
        }

        /**
         * Adds a option/argument to this command.
         * 
         * Each type of option has different settings you can adjust, but all of them have a "required" field that defaults
         * to true, you can set it to false for optional options.
         * 
         * @param name Name of the option, 1-32 characters (no symbols except - and _)
         * @param kind What type of option this is
         * @param description Description of the option, 1-100 characters long
         * @param opts Additional options, depends on what "kind" you pass but all options has a "required" field that defaults to true
         */
        addOption<TKey extends string, TKind extends OptionType, TRequired extends boolean | undefined>
            (name: TKey, kind: TKind, description: string, opts?: OptionsKindTable[TKind] & BaseOptionSettings<TRequired>) {

            let required = true;
            if (opts && opts.required !== undefined) {
                required = opts.required;
            }

            let fullOpts = {
                ...this.options,
                [name]: {
                    kind: kind,
                    required: required,
                    description: description,
                    extraOptions: {
                        ...opts,
                    }
                },
            }

            // Return a new builder with new typings
            // The new opts type is "layered" on top of the old one, making us able to use
            // the generic typings of all the options in the callback
            return new SlashCommandBuilder<LayerOption<TOpts, TKey, { kind: TKind, required: TRequired }>>
                (this.name, this.description, fullOpts, this.group);
        }

        /**
         * Build the command, providing a callback that runs when the command gets executed
         * @returns The built command, pass it to @{link Script.createCommand} to actually create it on discord 
         */
        build(callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<TOpts>) => void | Promise<any>): Command {
            return {
                name: this.name,
                description: this.description,
                kind: "Chat",
                options: this.options,
                group: this.group,
                ackMode: this.ackMode,
                cb: callback as any,
            };
        }
    }

    type LayerOption<TInner, TKey extends string, TVal> =
        { [Prop in keyof TInner]: TInner[Prop] } & { [Prop in TKey]: TVal };


    export interface BaseOptionSettings<TRequired extends boolean | undefined = undefined> {
        required?: TRequired
    }

    interface OptionsKindTable {
        Number: NumberOption,
        String: StringOption,
        Integer: IntegerOption,
        Boolean: BooleanOption,
        User: UserOption,
        Channel: ChannelOption,
        Role: RoleOption,
        Mentionable: MentionableOption,
    }

    type ParsedOptionsMap<T> = {
        [Prop in keyof T]: T[Prop] extends { required: false } ? (OptionParsedType<T[Prop]> | undefined) : OptionParsedType<T[Prop]>
    }

    type OptionParsedType<T> =
        T extends { kind: "String" } ? string :
        T extends { kind: "Number" } ? number :
        T extends { kind: "Integer" } ? number :
        T extends { kind: "Boolean" } ? boolean :
        T extends { kind: "User" } ? InteractionUser :
        T extends { kind: "Channel" } ? Internal.InteractionPartialChannel :
        T extends { kind: "Role" } ? DiscordModels.Role :
        T extends { kind: "Mentionable" } ? InteractionMentionable :
        unknown;

    /**
     * Creates a new user command builder. User commands show up in the context menu
     * when right-clicking a user
     * 
     * @param name 1-32 characters (no symbols except - and _)
     * @param description 1-100 characters
     */
    export function userCommand(name: string) {
        return new UserCommandBuilder(name);
    }

    export class UserCommandBuilder {
        name: string;
        ackMode: AckMode = "DeferredMessage";

        constructor(name: string) {
            this.name = name;
        }

        /**
         * Set the ack mode of this command
         * (if you're experienced with the discord api, this is the callback type to the interaction)
         * 
         * `DeferredMessage`: It will respond with a public deferred message.
         * `Custom`: You handle the ack'ing of the interaction yourself, this allows you to
         * use ephemeral responses and modals.
         * 
         * 
         * Keep in mind when using custom, you have to ack the interaction within 3 seconds otherwise it will fail.
         */
        setAckMode(mode: AckMode) {
            this.ackMode = mode;
            return this;
        }

        build(cb: (ctx: ExecutedCommandContext, target: InteractionUser) => any): Command {
            return {
                name: this.name,
                kind: "User",
                description: "",
                ackMode: this.ackMode,
                cb: cb as any,
            }
        }
    }

    /**
     * Creates a new message command builder. Message commands show up in the context menu
     * when right-clicking a message
     * 
     * @param name 1-32 characters (no symbols except - and _)
     * @param description 1-100 characters
     */
    export function messageCommand(name: string) {
        return new MessageCommandBuilder(name);
    }

    export class MessageCommandBuilder {
        name: string;
        ackMode: AckMode = "DeferredMessage";

        constructor(name: string) {
            this.name = name;
        }

        /**
         * Set the ack mode of this command
         * (if you're experienced with the discord api, this is the callback type to the interaction)
         * 
         * `DeferredMessage`: It will respond with a public deferred message.
         * `Custom`: You handle the ack'ing of the interaction yourself, this allows you to
         * use ephemeral responses and modals.
         * 
         * 
         * Keep in mind when using custom, you have to ack the interaction within 3 seconds otherwise it will fail.
         */
        setAckMode(mode: AckMode) {
            this.ackMode = mode;
            return this;
        }

        build(cb: (ctx: ExecutedCommandContext, target: DiscordModels.Message) => any): Command {
            return {
                name: this.name,
                kind: "Message",
                description: "",
                ackMode: this.ackMode,
                cb: cb as any,
            }
        }
    }
}