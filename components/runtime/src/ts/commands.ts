import { Discord, Events, Ops } from "./models";
import { EventMuxer } from "./events";
import { OpWrappers } from "./op_wrappers";
import { CommandInteractionOptionValue } from "./models/events/CommandInteractionOptionValue";
import { User } from "./models/discord/User";
import { PartialMember } from "./models/discord/PartialMember";
import { InteractionPartialMember } from "./models/events/InteractionPartialMember";
import { Role } from "./models/discord/Role";
import { InteractionPartialChannel } from "./models/events/InteractionChannel";

export namespace Commands {

    export interface CommandDef<T extends OptionsMap> {
        name: string;
        description: string;
        options: T;
        kind?: "chat" | "user" | "message";
        group?: Group,
        callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<T>) => void | Promise<any>,
    }

    export type OptionsMap = {
        [key: string]: BaseOption<boolean>;
    }

    type ParsedOptionsMap<T extends OptionsMap> = {
        [Property in keyof T]:
        T[Property] extends BaseOption<false> ? OptionTypeToParsedType<T[Property]> | undefined : OptionTypeToParsedType<T[Property]>;
    }

    interface BaseOption<TRequired extends boolean | undefined> {
        description: string;
        kind: OptionType;
        required?: TRequired;
    }

    export interface StringOption<T extends boolean> extends BaseOption<T> {
        kind: "String";
    };
    export interface NumberOption<T extends boolean> extends BaseOption<T> {
        kind: "Number";
    };
    export interface IntOption<T extends boolean> extends BaseOption<T> {
        kind: "Integer";
    };
    export interface BoolOption<T extends boolean> extends BaseOption<T> {
        kind: "Boolean";
    };
    export interface UserOption<T extends boolean> extends BaseOption<T> {
        kind: "User";
    };
    export interface ChannelOption<T extends boolean> extends BaseOption<T> {
        kind: "Channel";
    };
    export interface RoleOption<T extends boolean> extends BaseOption<T> {
        kind: "Role";
    };
    export interface MentionableOption<T extends boolean> extends BaseOption<T> {
        kind: "Mentionable";
    };

    export type OptionType = Ops.CommandOptionType;

    type OptionTypeToParsedType<T extends BaseOption<boolean>> =
        T extends StringOption<boolean> ? string :
        T extends NumberOption<boolean> ? number :
        T extends IntOption<boolean> ? number :
        T extends BoolOption<boolean> ? boolean :
        T extends UserOption<boolean> ? InteractionUser :
        T extends ChannelOption<boolean> ? InteractionPartialChannel :
        T extends RoleOption<boolean> ? Role :
        T extends MentionableOption<boolean> ? InteractionMentionable :
        unknown;

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

    export class System {
        commands: CommandDef<OptionsMap>[] = [];

        addEventListeners(muxer: EventMuxer) {
            muxer.on("BOTLOADER_COMMAND_INTERACTION_CREATE", this.handleInteractionCreate.bind(this));
        }

        async handleInteractionCreate(interaction: Events.CommandInteraction) {
            let command = this.commands.find(cmd => matchesCommand(cmd, interaction));
            if (!command) {
                return;
            }

            let optionsMap: Record<string, any> = {};
            for (let opt of interaction.options) {
                optionsMap[opt.name] = this.resolveOption(interaction.dataMap, opt.value);
            }
            await command.callback(new ExecutedCommandContext(interaction), optionsMap)
        }

        private resolveOption(map: Events.CommandInteractionDataMap, opt: CommandInteractionOptionValue): unknown {
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
                    const channel: InteractionPartialChannel = map.channels[opt.value];
                    if (channel === undefined) {
                        throw new Error("interaction channel not found in data map");
                    }
                    return channel;

                default:
                    return opt.value;
            }
        }

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

    function matchesCommand(cmd: CommandDef<any>, interaction: Events.CommandInteraction) {
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
        interaction: Events.CommandInteraction;

        constructor(interaction: Events.CommandInteraction) {
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
}

export interface InteractionUser {
    user: User,
    member?: InteractionPartialMember,
}

export type InteractionMentionable = {
    kind: "Role",
    value: Role
} | {
    kind: "User",
    value: InteractionUser
}