import { Events, Ops, DiscordModels, Internal } from "./generated";
import { EventMuxer } from "./events";
import { OpWrappers } from "./op_wrappers";

export namespace Commands {
    export class System {
        commands: Command[] = [];

        addEventListeners(muxer: EventMuxer) {
            muxer.on("BOTLOADER_COMMAND_INTERACTION_CREATE", this.handleInteractionCreate.bind(this));
        }

        async handleInteractionCreate(interaction: Internal.CommandInteraction) {
            let command = this.commands.find(cmd => matchesCommand(cmd, interaction));
            if (!command) {
                return;
            }

            let optionsMap: Record<string, any> = {};
            for (let opt of interaction.options) {
                optionsMap[opt.name] = this.resolveOption(interaction.dataMap, opt.value);
            }
            await command.cb(new ExecutedCommandContext(interaction), optionsMap)
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
                    const channel: Events.InteractionPartialChannel = map.channels[opt.value];
                    if (channel === undefined) {
                        throw new Error("interaction channel not found in data map");
                    }
                    return channel;

                default:
                    return opt.value;
            }
        }

        /**
         * @internal
         */
        genOpBinding(): [Ops.Command[], Ops.CommandGroup[]] {

            const commands: Ops.Command[] = this.commands.map(cmd => {
                const options: Ops.CommandOption[] = [];
                for (let prop in cmd.options) {
                    if (Object.prototype.hasOwnProperty.call(cmd.options, prop)) {
                        let entry = cmd.options[prop];
                        options.push({
                            name: prop,
                            description: entry.description,
                            kind: entry.kind,
                            required: entry.required || false,
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
                    group,
                    subGroup,
                }
            });

            const groups: Ops.CommandGroup[] = [];

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

    function matchesCommand(cmd: Command, interaction: Internal.CommandInteraction) {
        if (interaction.parentParentName) {
            if (cmd.group && cmd.group.parent) {
                return cmd.name === interaction.name && cmd.group.name === interaction.parentName && cmd.group.parent.name === interaction.parentParentName;
            }
        } else if (interaction.parentName) {
            if (cmd.group && !cmd.group.parent) {
                return cmd.name === interaction.name && cmd.group.name === interaction.parentName;
            }
        } else {
            if (!cmd.group) {
                return cmd.name === interaction.name;
            }
        }
    }

    export class ExecutedCommandContext {
        interaction: Internal.CommandInteraction;

        constructor(interaction: Internal.CommandInteraction) {
            this.interaction = interaction;
        }

        async sendResponse(resp: string | Ops.OpCreateMessageFields) {
            if (typeof resp === "string") {
                await OpWrappers.createInteractionFollowup({
                    interactionToken: this.interaction.token,
                    fields: { content: resp }
                })
            } else {
                await OpWrappers.createInteractionFollowup({
                    interactionToken: this.interaction.token,
                    fields: resp
                })
            }
        }
    }


    export interface InteractionUser {
        user: DiscordModels.User,
        member?: Events.InteractionPartialMember,
    }

    export type InteractionMentionable = {
        kind: "Role",
        value: DiscordModels.Role
    } | {
        kind: "User",
        value: InteractionUser
    }

    export interface Command {
        name: string;
        description: string;
        kind: "Chat" | "User" | "Message",
        group?: Group,
        options?: OptionMap;
        cb: (ctx: {}, args: {}) => any,
    }

    export type OptionType = BaseOption["kind"];
    export type Option = BaseOption & (StringOption | NumberOption);

    export type AutocompleteProvider<T> = (data: {}) => Promise<OptionChoice<T>[]> | OptionChoice<T>[];

    export type OptionMap = {
        [key: string]: Option,
    }

    export interface BaseOption {
        kind: "String" | "Number" | "Integer" | "Boolean" | "User" | "Channel" | "Role" | "Mentionable",
        description: string,
        required: boolean,
    }

    export interface StringOption {
        choices?: OptionChoice<string>[],
        autocomplete?: AutocompleteProvider<string>,
    }

    export interface NumberOption {
        choices?: OptionChoice<number>[],
        min_value?: number,
        max_value?: number,
        autocomplete?: AutocompleteProvider<number>,
    }

    export interface IntegerOption {
        choices?: OptionChoice<number>[],
        min_value?: number,
        max_value?: number,
        autocomplete?: AutocompleteProvider<number>,
    }

    export interface BooleanOption {

    }

    export interface UserOption {

    }

    export interface ChannelOption {
        channel_types?: DiscordModels.ChannelType[],
    }

    export interface RoleOption {

    }

    export interface MentionableOption {

    }

    export interface OptionChoice<T> {
        name: string,
        value: T,
    }

    export class Group {
        name: string;
        description: string;
        parent?: Group;
        protected isSubGroup: boolean = false;

        subGroups: Group[] = [];

        constructor(name: string, description: string) {
            this.name = name;
            this.description = description;
        }

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

    export function slashCommand(name: string, description: string) {
        return new SlashCommandBuilder<{}>(name, description, {});
    }

    class SlashCommandBuilder<TOpts> {
        private name: string;
        private description: string;
        private options: OptionMap;
        private group?: Group;

        constructor(name: string, description: string, options: OptionMap, group?: Group) {
            this.name = name;
            this.description = description;
            this.options = options;
        }

        addOptionNumber<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: NumberOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Number", description, opts)
        }

        addOptionString<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: StringOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "String", description, opts)
        }

        addOptionInteger<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: IntegerOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Integer", description, opts)
        }

        addOptionBoolean<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: BooleanOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Boolean", description, opts)
        }

        addOptionUser<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: UserOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "User", description, opts)
        }

        addOptionChannel<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: ChannelOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Channel", description, opts)
        }

        addOptionRole<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: RoleOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Role", description, opts)
        }

        addOptionMentionable<TKey extends string, TRequired extends boolean | undefined>
            (key: TKey, description: string, opts?: MentionableOption & BaseOptionSettings<TRequired>) {
            return this.addOption(key, "Mentionable", description, opts)
        }

        addOption<TKey extends string, TKind extends OptionType, TRequired extends boolean | undefined>
            (key: TKey, kind: TKind, description: string, opts?: OptionsKindTable[TKind] & BaseOptionSettings<TRequired>) {

            let required = false;
            if (opts && opts.required !== undefined) {
                required = true;
            }

            let fullOpts = {
                [key]: {
                    kind: kind,
                    required: required,
                    description: description,
                    ...opts
                },
                ...this.options,
            }

            // Return a new builder with new typings
            // The new opts type is "layered" on top of the old one, making us able to use
            // the generic typings of all the options in the callback
            return new SlashCommandBuilder<LayerOption<TOpts, TKey, { kind: TKind, required: TRequired }>>
                (this.name, this.description, fullOpts as OptionMap);
        }

        build(callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<TOpts>) => void | Promise<any>): Command {
            return {
                name: this.name,
                description: this.description,
                kind: "Chat",
                options: this.options,
                group: this.group,
                cb: callback as any,
            };
        }
    }

    type LayerOption<TInner, TKey extends string, TVal> =
        { [Prop in keyof TInner]: TInner[Prop] } & { [Prop in TKey]: TVal };


    interface BaseOptionSettings<TRequired extends boolean | undefined = undefined> {
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
        T extends { kind: "Channel" } ? Events.InteractionPartialChannel :
        T extends { kind: "Role" } ? DiscordModels.Role :
        T extends { kind: "Mentionable" } ? InteractionMentionable :
        unknown;

    export function userCommand(name: string, description: string) {
        return new UserCommandBuilder(name, description);
    }

    class UserCommandBuilder {
        name: string;
        description: string;

        constructor(name: string, description: string) {
            this.name = name;
            this.description = description;
        }

        build(cb: (ctx: ExecutedCommandContext, target: InteractionUser) => any): Command {
            return {
                name: this.name,
                kind: "User",
                description: this.description,
                cb: cb as any,
            }
        }
    }

    export function messageCommand(name: string, description: string) {
        return new MessageCommandBuilder(name, description);
    }

    class MessageCommandBuilder {
        name: string;
        description: string;

        constructor(name: string, description: string) {
            this.name = name;
            this.description = description;
        }

        build(cb: (ctx: ExecutedCommandContext, target: DiscordModels.Message) => any): Command {
            return {
                name: this.name,
                kind: "Message",
                description: this.description,
                cb: cb as any,
            }
        }
    }
}