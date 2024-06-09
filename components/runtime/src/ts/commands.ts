import * as Internal from "./generated/internal/index";
import { ChannelType, Interaction, Message, Role, IModalFields, MessageFlags, AutocompleteInteraction } from "./discord/index";
import { User } from "./discord/user";
import { Member } from "./discord/member";
import { OpWrappers } from "./op_wrappers";

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

            if (interaction.isAutocomplete) {
                await this.handleAutoComplete(command, interaction)
                return
            }

            let ctx = new ExecutedCommandContext(interaction);
            if (command.ackMode === "DeferredMessage") {
                await ctx.ackWithDeferredMessage({
                    flags: command.ackMessageFlags,
                });
            }

            if (interaction.kind === "Chat") {
                let optionsMap: Record<string, any> = {};
                for (let opt of interaction.options) {
                    optionsMap[opt.name] = this.resolveOption(interaction.dataMap, opt.value);
                }

                await this.runCommand(command, ctx, optionsMap)
            } else if (interaction.kind === "Message") {
                if (interaction.targetId) {
                    let message = interaction.dataMap.messages[interaction.targetId];
                    await this.runCommand(command, ctx, new Message(message))
                } else {
                    throw new Error("message not found in datamap")
                }
            } else if (interaction.kind === "User") {
                if (interaction.targetId) {
                    let member = interaction.dataMap.members[interaction.targetId];
                    let user = interaction.dataMap.users[interaction.targetId]
                    let args: InteractionUser = {
                        user: new User(user),
                        member,
                    };

                    await this.runCommand(command, ctx, args)
                } else {
                    throw new Error("member not found in datamap")
                }
            } else {
                throw new Error("Unknown command type")
            }
        }

        async runCommand(command: Command, ctx: ExecutedCommandContext, args: any) {
            try {
                await command.cb(ctx, args)

                const fullName = fullCommandName(ctx.commandName, ctx.parentName, ctx.parentParentName)
                if (!ctx.hasSentCallback) {
                    ctx.ackWithMessage({
                        embeds: [{
                            title: "Script Error",
                            description: "Command handler returned without acking the interaction.",
                            color: 0xdb4444,
                        }],
                        flags: {
                            ephemeral: true,
                        }
                    })

                    console.output({
                        items: [`Command handler [${fullName}] did not send a callback before it returned.`],
                        level: "error",
                        includeCaller: false
                    })
                } else if (ctx.isResponseDeferred && !ctx.isDeferredResponseSent) {
                    ctx.createFollowup({
                        embeds: [{
                            title: "Script Error",
                            description: "Command handler returned without sending a response.",
                            color: 0xdb4444,
                        }],
                        flags: {
                            ephemeral: true,
                        }
                    })

                    console.output({
                        items: [`Command handler [${fullName}] did not send a response before it returned.`],
                        level: "error",
                        includeCaller: false
                    })
                }
            } catch (error: any) {
                let origStack = error.stack

                if (ctx.hasSentCallback) {
                    ctx.createFollowup({
                        embeds: [{
                            title: "Script Error",
                            description: `Command handler threw an exception, the error has been logged to the console for admins to view.`,
                            color: 0xdb4444,
                        }],
                        flags: {
                            ephemeral: true,
                        }
                    })

                } else {
                    ctx.ackWithMessage({
                        embeds: [{
                            title: "Script Error",
                            description: `Command handler threw an exception, the error has been logged to the console for admins to view.`,
                            color: 0xdb4444,
                        }],
                        flags: {
                            ephemeral: true,
                        }
                    })
                }

                const fullName = fullCommandName(ctx.commandName, ctx.parentName, ctx.parentParentName)
                console.output({
                    items: [`Command handler [${fullName}] threw an exception: ${error.message}\n${origStack}`],
                    level: "error",
                    includeCaller: false
                })
            }
        }

        async handleAutoComplete(command: Command, interaction: Internal.CommandInteraction) {
            let options: OptionChoice<number | string>[] = []
            try {
                options = await this.getAutocompleteResponse(command, interaction)
            } catch (error) {
                console.log("Autocomplete handler threw an error: ", error)
                options = []
            }

            await OpWrappers.interactionCallback({
                interactionId: interaction.id,
                interactionToken: interaction.token,
                data: {
                    kind: "Autocomplete",
                    choices: options,
                }
            })
        }

        async getAutocompleteResponse(command: Command, interaction: Internal.CommandInteraction): Promise<OptionChoice<string>[] | OptionChoice<number>[]> {
            const focusedInteractionOption = interaction.options.find(v => v.value.kind === "focused")
            if (!focusedInteractionOption || focusedInteractionOption.value.kind !== "focused") {
                console.log("Could not find focused option in autocomplete interaction")
                return []
            }

            const interactionArg = new AutocompleteInteraction(interaction, focusedInteractionOption.value.value)
            const definition = command.options ? command.options[focusedInteractionOption.name] : null
            if (!definition) {
                console.log("Could not find option deifnition for focused option ", { command: command.name, option: focusedInteractionOption.name })
                return []
            }

            if (!("autocomplete" in definition.extraOptions) || !definition.extraOptions.autocomplete) {
                console.log("Command option does not have autocomplete ", { command: command.name, option: focusedInteractionOption.name })
                return []
            }

            const optionsOutput = await definition.extraOptions.autocomplete(interactionArg)
            return optionsOutput
        }

        private resolveOption(map: Internal.CommandInteractionDataMap, opt: Internal.CommandInteractionOptionValue): unknown {
            switch (opt.kind) {
                case "user":
                    const user = map.users[opt.value];
                    if (user === undefined) {
                        throw new Error("interaction user not found in data map");
                    }

                    const ret: InteractionUser = {
                        user: new User(user),
                        member: map.members[opt.value],
                    }
                    return ret

                case "role":
                    const role: Role = map.roles[opt.value];
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
                                user: new User(mentionableUser),
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
            super(interaction.id, interaction.token, new Member(interaction.member));

            this.channelId = interaction.channelId;
            this.commandName = interaction.name;
            this.parentName = interaction.parentName ?? undefined;
            this.parentParentName = interaction.parentParentName ?? undefined;
        }

        /**
         * Acknowledge this interaction and open up a modal for the user.
         * 
         * You have to acknowledge the interaction within 3 seconds, and it can only be done once. 
         */
        async ackWithModal(modal: IModalFields) {
            this.setCallbackSent();

            return OpWrappers.interactionCallback({
                interactionId: this.interactionId,
                interactionToken: this.token,
                data: {
                    kind: "Modal",
                    title: modal.title,
                    customId: modal.customId,
                    components: modal.components,
                }
            })
        }
    }

    export interface InteractionUser {
        user: User,
        member?: Internal.InteractionPartialMember,
    }

    export type InteractionMentionable = {
        kind: "Role",
        value: Role
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
        ackMessageFlags: MessageFlags,
        cb: (ctx: {}, args: {}) => any,
    }

    export type OptionType = Option["kind"];

    export type AutocompleteProvider<T> = (interaction: AutocompleteInteraction) => Promise<OptionChoice<T>[]> | OptionChoice<T>[];

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
        /**
         * A list of choices to present the user, up to 25 entries.
         */
        choices?: OptionChoice<string>[],

        /**
         * Enables autocomplete for this option.
         * 
         * The callback will be called as users type and you respond with options that will show up on the fly.
         */
        autocomplete?: AutocompleteProvider<string>,
    }

    export interface NumberOption {
        /**
         * A list of choices to present the user, up to 25 entries.
         */
        choices?: OptionChoice<number>[],

        minValue?: number,
        maxValue?: number,

        /**
         * Enables autocomplete for this option.
         * 
         * The callback will be called as users type and you respond with options that will show up on the fly.
         */
        autocomplete?: AutocompleteProvider<number>,
    }

    export interface IntegerOption {
        /**
         * A list of choices to present the user, up to 25 entries.
         */
        choices?: OptionChoice<number>[],

        minValue?: number,
        maxValue?: number,

        /**
         * Enables autocomplete for this option.
         * 
         * The callback will be called as users type and you respond with options that will show up on the fly.
         */
        autocomplete?: AutocompleteProvider<number>,
    }

    export interface BooleanOption {

    }

    export interface UserOption {

    }

    export interface ChannelOption {
        channelTypes?: ChannelType[],
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
            group.parent = this;
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
     * @example 
     * ```ts
     * script.createSlashCommand("echo", "echo's your input")
     * .addOptionString("what", "what to echo")
     * .build(async (ctx, args) => {
     *      await ctx.sendResponse(args.what);
     * })
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
        private ackMessageFlags: MessageFlags = {}

        /**
         * @internal
         */
        onBuilt?: (cmd: Command) => void

        constructor(name: string, description: string, options: OptionMap, group?: Group) {
            this.name = name;
            this.description = description;
            this.options = options;
            this.group = group;
            this.onBuilt = undefined;
        }

        /**
         * Assigns a group to this command
         * 
         * @example 
         * ```ts
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
         * `DeferredMessage`: It will respond with a deferred message, using the flags from setAckMessageFlags.
         * `Custom`: You handle the ack'ing of the interaction yourself, this allows you to use modals.
         * 
         * Keep in mind when using custom, you have to ack the interaction within 3 seconds otherwise it will fail.
         */
        setAckMode(mode: AckMode) {
            this.ackMode = mode;
            return this;
        }

        /**
         * Sets the flags for the mssage sent when sending the intial interaction response.
         * Epehemeral means that it can only be see by the person issuing the command.
         */
        setAckMessageFlags(flags: Pick<MessageFlags, "ephemeral" | "suppressEmbeds">) {
            this.ackMessageFlags = flags
            return this
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
            let next = new SlashCommandBuilder<LayerOption<TOpts, TKey, { kind: TKind, required: TRequired }>>
                (this.name, this.description, fullOpts, this.group);
            Object.assign(next, this);
            next.options = fullOpts;

            return next;
        }

        /**
         * Build the command, providing a callback that runs when the command gets executed
         * @returns The built command, if you used `script.createSlashCommand` you can ignore the return value but if not you pass it to @{link Script.createCommand} to actually create it on discord 
         */
        build(callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<TOpts>) => void | Promise<any>): Command {
            const built: Command = {
                name: this.name,
                description: this.description,
                kind: "Chat",
                options: this.options,
                group: this.group,
                ackMode: this.ackMode,
                ackMessageFlags: this.ackMessageFlags,
                cb: callback as any,
            };

            if (this.onBuilt) {
                this.onBuilt(built)
            }

            return built
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
        T extends { kind: "Role" } ? Role :
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
        private ackMessageFlags: MessageFlags = {}

        /**
         * @internal
         */
        onBuilt?: (cmd: Command) => void

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

        /**
         * Sets the flags for the mssage sent when sending the intial interaction response.
         * Epehemeral means that it can only be see by the person issuing the command.
         */
        setAckMessageFlags(flags: Pick<MessageFlags, "ephemeral" | "suppressEmbeds">) {
            this.ackMessageFlags = flags
            return this
        }

        build(cb: (ctx: ExecutedCommandContext, target: InteractionUser) => any): Command {
            const built: Command = {
                name: this.name,
                kind: "User",
                description: "",
                ackMode: this.ackMode,
                cb: cb as any,
                ackMessageFlags: this.ackMessageFlags,
            }

            if (this.onBuilt) {
                this.onBuilt(built)
            }

            return built
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
        private ackMessageFlags: MessageFlags = {}

        /**
         * @internal
         */
        onBuilt?: (cmd: Command) => void

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

        /**
         * Sets the flags for the mssage sent when sending the intial interaction response.
         * Epehemeral means that it can only be see by the person issuing the command.
         */
        setAckMessageFlags(flags: Pick<MessageFlags, "ephemeral" | "suppressEmbeds">) {
            this.ackMessageFlags = flags
            return this
        }

        build(cb: (ctx: ExecutedCommandContext, target: Message) => any): Command {
            const built: Command = {
                name: this.name,
                kind: "Message",
                description: "",
                ackMode: this.ackMode,
                cb: cb as any,
                ackMessageFlags: this.ackMessageFlags,
            }

            if (this.onBuilt) {
                this.onBuilt(built)
            }

            return built
        }
    }
}

function fullCommandName(commandName: string, parentName: string | undefined, parentParentName: string | undefined) {
    let parent = parentName ? (parentName + " ") : ""
    let parentParent = parentParentName ? (parentParentName + " ") : ""
    return "/" + parentParent + parent + commandName
}