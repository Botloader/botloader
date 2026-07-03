import { ChannelType, ComponentType, IComponent, Role, ThreadMetadata, Attachment } from '../generated/discord/index';
import * as Internal from '../generated/internal/index';
import {
    CreateMessageFields,
    InteractionMessageFlags,
    InteractionCreateMessageFields,
    createInteractionFollowupMessage,
    getInteractionFollowupMessage,
    deleteInteractionOriginalResponse,
    editInteractionOriginalResponse,
    getInteractionOriginalResponse,
    editInteractionFollowupMessage,
    deleteInteractionFollowupMessage,
    toOpMessageFields
} from './dapi';
import { OpWrappers } from '../op_wrappers';
import { Member } from './member';
import { Message } from './message';
import { GuildChannel } from './channel';
import { User } from './user';

/**
 * Base interaction class, this class should be considered UNSTABLE and may change a lot in the future.
 */
export class Interaction {
    interactionId: string;
    token: string;

    /**
     * The user that started the interaction
     */
    member: Member;

    protected _hasSentCallback = false;
    protected _isResponseDeferred = false;
    protected _isDeferredResponseSent = false;

    get hasSentCallback() {
        return this._hasSentCallback;
    }

    get isResponseDeferred() {
        return this._isResponseDeferred;
    }

    get isDeferredResponseSent() {
        return this._isDeferredResponseSent;
    }

    constructor(id: string, token: string, member: Member) {
        this.interactionId = id;
        this.member = member;
        this.token = token;
    }

    protected setCallbackSent() {
        if (this.hasSentCallback) {
            throw new Error("tried sending a callback when one has already been sent, only one callback per interaction can be sent.")
        } else {
            this._hasSentCallback = true;
        }
    }

    /**
     * @deprecated use {@link ackWithDeferredMessage} 
     */
    async sendCallbackWithDeferredMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.ackWithDeferredMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * Acknowledge this interaction and send a message in response to this interaction
     */
    async ackWithMessage(fields: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "ChannelMessageWithSource",
                fields: toOpMessageFields(fields),
                flags: fields.flags || {},
            }
        })
    }

    /**
     * Acknowledge this interaction and display a "thinking" state to the user for you to then send a followUp 
     * message later.
     * 
     * You have to ack interactions within 3 seconds but if you are doing things that can take longer than that you can
     * use this function first to tell discord that you are processing the interaction then send the message itself later.
     */
    async ackWithDeferredMessage(fields?: InteractionCreateMessageFields) {
        this.setCallbackSent();
        this._isResponseDeferred = true

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "DeferredChannelMessageWithSource",
                fields: toOpMessageFields(fields ?? {}),
                flags: fields?.flags ?? {},
            }
        })
    }

    /**
     * @deprecated use {@link createFollowup} instead
     */
    async sendResponse(resp: string | CreateMessageFields) {
        return this.createFollowup(resp);
    }

    async getOriginalResponse() {
        return getInteractionOriginalResponse(this.token);
    }

    async editOriginalResponse(fields: InteractionCreateMessageFields) {
        this._isDeferredResponseSent = true
        return editInteractionOriginalResponse(this.token, fields)
    }

    async deleteOriginalResponse() {
        this._isDeferredResponseSent = true
        return deleteInteractionOriginalResponse(this.token);
    }

    async getFollowup(messageId: string) {
        return getInteractionFollowupMessage(this.token, messageId);
    }

    /**
     * @deprecated use {@link createFollowup} instead
     */
    async sendFollowup(resp: string | InteractionCreateMessageFields) {
        this._isDeferredResponseSent = true
        return createInteractionFollowupMessage(this.token, resp);
    }

    async createFollowup(resp: string | InteractionCreateMessageFields) {
        this._isDeferredResponseSent = true
        return createInteractionFollowupMessage(this.token, resp);
    }

    async editFollowup(messageId: string, fields: InteractionCreateMessageFields) {
        return editInteractionFollowupMessage(this.token, messageId, fields);
    }

    async deleteFollowup(id: string) {
        return deleteInteractionFollowupMessage(this.token, id);
    }
}

export class ComponentInteraction extends Interaction {
    customIdRaw: string;
    componentType: ComponentType;
    channelId: string;
    message: Message;

    /**
     * @internal
     */
    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction.id, interaction.token, new Member(interaction.member));

        this.componentType = interaction.componentType;
        this.customIdRaw = interaction.customId;
        this.channelId = interaction.channelId;
        this.message = new Message(interaction.message);
    }

    /**
     * @deprecated use {@link ackWithUpdateMessage}
     */
    async sendCallbackUpdateMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        return this.ackWithUpdateMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * @deprecated use {@link ackWithDeferredUpdateMessage}
     */
    async sendCallbackDeferredUpdateMessage() {
        return this.ackWithDeferredUpdateMessage();
    }

    /**
     * Acknowledge this interaction and update the message the component was on
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithUpdateMessage(fields: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "UpdateMessage",
                fields: toOpMessageFields(fields),
                flags: fields.flags || {},
            }
        })
    }

    /**
     * Acknowledge this interaction and update the message the component was on at a later time (within 15 mins).
     * 
     * You have to ack interactions within 3 seconds but if you are doing things that can take longer than that you can
     * use this function first to tell discord that you are processing the interaction then update the message later.
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithDeferredUpdateMessage() {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "DeferredUpdateMessage",
            }
        })
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

export interface IModalFields {
    title: string,
    customId: string,
    components: IComponent[],
}

export class ModalSubmitInteraction extends Interaction {
    customIdRaw: string;
    channelId: string;
    message: Message | null;

    /**
     * Raw access to this values array has been superseded and will now cause a typing error. Please migrate to the newer accessor methods: {@link getTextInputValue}, etc...
     */
    values: ModalSubmitInteractionValues;

    /**
     * @internal
     */
    constructor(interaction: Internal.IModalInteraction) {
        super(interaction.id, interaction.token, new Member(interaction.member));

        this.customIdRaw = interaction.customId;
        this.channelId = interaction.channelId;
        this.message = interaction.message ? new Message(interaction.message) : null;

        this.values = {};

        for (const elem of interaction.values) {
            this.parseComponentValues(elem, interaction);
        }
    }

    private parseComponentValues(elem: Internal.ModalInteractionComponent, interaction: Internal.IModalInteraction) {
        switch (elem.kind) {
            // Handle the components that have nested components
            case "Label":
                this.parseComponentValues(elem.component, interaction);
                break;

            case "ActionRow":
                for (const subElem of elem.components) {
                    this.parseComponentValues(subElem, interaction);
                }
                break;
            
            // Value components
            case 'SelectMenu': {
                const parsed = new ModalSubmitInteractionValueSelectMenu(elem);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'ChannelSelectMenu': {
                const parsed = new ModalSubmitInteractionValueChannelSelectMenu(elem, interaction.resolved);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'UserSelectMenu': {
                const parsed = new ModalSubmitInteractionValueUserSelectMenu(elem, interaction.resolved);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'RoleSelectMenu': {
                const parsed = new ModalSubmitInteractionValueRoleSelectMenu(elem, interaction.resolved);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'MentionableSelectMenu': {
                const parsed = new ModalSubmitInteractionValueMentionableSelectMenu(elem, interaction.resolved);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'FileUpload': {
                const parsed = new ModalSubmitInteractionValueFileUpload(elem, interaction.resolved);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'TextInput': {
                const parsed = new ModalSubmitInteractionValue(elem);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'Checkbox': {
                const parsed = new ModalSubmitInteractionValueCheckbox(elem);
                this.values[parsed.name] = parsed;
                break;
            }
            case 'CheckboxGroup': {
                const parsed = new ModalSubmitInteractionValueCheckboxGroup(elem);
                this.values[parsed.name] = parsed;
                break;
            }
        }
    }

    private getComponentValue<T>(name: string, kind: ComponentType): T | null {
        if (!(name in this.values)) {
            return null;
        }

        const value = this.values[name];
        return value.kind === kind ? value as T : null;
    }

    /**
     * Gets a text input value
     * @returns The component value if the component both exists and is a text input, null otherwise
     */
    getTextInputValue(name: string): ModalSubmitInteractionValue | null {
        return this.getComponentValue(name, "TextInput");
    }

    /**
     * Gets a select menu value 
     * @returns The component value if the component both exists and is a select menu, null otherwise
     */
    getSelectMenuValue(name: string) : ModalSubmitInteractionValueSelectMenu | null {
        return this.getComponentValue(name, "SelectMenu");
    }

    /**
     * Gets a channel select menu value 
     * @returns The component value if the component both exists and is a channel select menu, null otherwise
     */
    getChannelSelectMenuValue(name: string) : ModalSubmitInteractionValueChannelSelectMenu | null {
        return this.getComponentValue(name, "ChannelSelectMenu");
    }

    /**
     * Gets a role select menu value 
     * @returns The component value if the component both exists and is a role select menu, null otherwise
     */
    getRoleSelectMenuValue(name: string): ModalSubmitInteractionValueRoleSelectMenu | null {
        return this.getComponentValue(name, "RoleSelectMenu");
    }

    /**
     * Gets a user select menu value 
     * @returns The component value if the component both exists and is a user select menu, null otherwise
     */
    getUserSelectMenuValue(name: string): ModalSubmitInteractionValueUserSelectMenu | null {
        return this.getComponentValue(name, "UserSelectMenu");
    }

    /**
     * Gets a mentionable select menu value 
     * @returns The component value if the component both exists and is a mentionable select menu, null otherwise
     */
    getMentionableSelectMenuValue(name: string): ModalSubmitInteractionValueMentionableSelectMenu | null {
        return this.getComponentValue(name, "MentionableSelectMenu");
    }

    /**
     * Gets a checkbox value 
     * @returns The component value if the component both exists and is a checkbox, null otherwise
     */
    getCheckboxValue(name: string): ModalSubmitInteractionValueCheckbox | null {
        return this.getComponentValue(name, "Checkbox");
    }

    /**
     * Gets the checkbox group values 
     * @returns The component values if the component both exists and is a checkbox group, null otherwise
     */
    getCheckboxGroupValue(name: string): ModalSubmitInteractionValueCheckboxGroup | null {
        return this.getComponentValue(name, "CheckboxGroup");
    }
    
    /**
     * Gets the file upload value 
     * @returns The component value if the component both exists and is a checkbox, null otherwise
     */
    getFileUploadValue(name: string): ModalSubmitInteractionValueFileUpload | null {
        return this.getComponentValue(name, "FileUpload");
    }

    /**
     * @deprecated use {@link ackWithUpdateMessage}
     */
    async sendCallbackUpdateMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        return this.ackWithUpdateMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * @deprecated use {@link ackWithDeferredUpdateMessage}
     */
    async sendCallbackDeferredUpdateMessage() {
        return this.ackWithDeferredUpdateMessage();
    }

    /**
     * Acknowledge this interaction and update the message the component was on
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithUpdateMessage(fields: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "UpdateMessage",
                fields: toOpMessageFields(fields),
                flags: fields.flags || {},
            }
        })
    }

    /**
     * Acknowledge this interaction and update the message the component was on at a later time (within 15 mins).
     * 
     * You have to ack interactions within 3 seconds but if you are doing things that can take longer than that you can
     * use this function first to tell discord that you are processing the interaction then update the message later.
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithDeferredUpdateMessage() {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            interactionToken: this.token,
            data: {
                kind: "DeferredUpdateMessage",
            }
        })
    }
}

export interface InteractionMember {
    joinedAt: number;
    nick: string | null;
    premiumSince?: number;
    roles: Array<string>;
}

export interface InteractionUser {
    user: User,
    member?: InteractionMember,
}

export type InteractionMentionable = {
    kind: "Role",
    value: Role
} | {
    kind: "User",
    value: InteractionUser
}

/**
 * Helper class to resolve interaction values to their corresponding types
 * 
 * @internal
 */
class InteractionDataResolver {
    static fromRoleSelectMenu(values: string[], resolved: Internal.InteractionDataMap | null) {
        const roleValues: Role[] = [];

        if (!resolved) {
            throw new Error("No roles resolved in data, this is a bot error you should report to the botloader team.")
        }

        for (const id of values) {
            const role = resolved.roles[id];
            if (!role) {
                throw new Error("Failed to resolve role: " + role)
            }
            roleValues.push(role);
        }

        return roleValues;
    }

    static fromChannelSelectMenu(values: string[], resolved: Internal.InteractionDataMap | null) {
        const channelValues: InteractionChannel[] = [];

        if (!resolved) {
            throw new Error("No resolved data, this is a bot error you should report to the botloader team.")
        }

        for (const id of values) {
            const channel = resolved.channels[id]
            if (!channel) {
                throw new Error("Failed to resolve channel: " + id)
            }
            channelValues.push(channel)
        }

        return channelValues
    }

    static fromUserSelectMenu(values: string[], resolved: Internal.InteractionDataMap | null) {
        const userValues: InteractionUser[] = [];

        if (!resolved) {
            throw new Error("No users resolved in data, this is a bot error you should report to the botloader team.")
        }

        for (const id of values) {
            const user = resolved.users[id]
            if (!user) {
                throw new Error("Failed to resolve user: " + id)
            }

            userValues.push({
                user: new User(user),
                member: resolved.members[id],
            })
        }

        return userValues;
    }

    static fromMentionablesSelectMenu(values: string[], resolved: Internal.InteractionDataMap | null) {
        const mentionableValues: InteractionMentionable[] = [];
        
        if (!resolved) {
            throw new Error("No mentionables resolved in data, this is a bot error you should report to the botloader team.")
        }

        for (const id of values) {
            const role = resolved.roles[id]
            if (role) {
                mentionableValues.push({
                    kind: "Role",
                    value: role
                })

                continue;
            }

            const user = resolved.users[id]
            if (!user) {
                throw new Error("Failed to resolve mentionable: " + id)
            }

            mentionableValues.push({
                kind: "User",
                value: {
                    user: new User(user),
                    member: resolved.members[id],
                }
            })
        }

        return mentionableValues;
    }

    static fromAttachmentsUpload(values: string[], resolved: Internal.InteractionDataMap | null) {
        const attachmentValues: Attachment[] = [];
        if (!resolved) {
            throw new Error("No attachments resolved in data, this is a bot error you should report to the botloader team.")
        }

        for (const id of values) {
            const attachment = resolved.attachments[id]
            if (!attachment) {
                throw new Error("Failed to resolve attachment: " + id)
            }

            attachmentValues.push(attachment)
        }

        return attachmentValues;
    }
}

export interface ModalSubmitInteractionValues {
    [key: string]: ModalSubmitInteractionValue
                  | ModalSubmitInteractionValueSelectMenu
                  | ModalSubmitInteractionValueChannelSelectMenu
                  | ModalSubmitInteractionValueRoleSelectMenu
                  | ModalSubmitInteractionValueUserSelectMenu
                  | ModalSubmitInteractionValueMentionableSelectMenu
                  | ModalSubmitInteractionValueCheckbox
                  | ModalSubmitInteractionValueCheckboxGroup
                  | ModalSubmitInteractionValueFileUpload;
}

class ModalSubmitInteractionValueBase {
    customIdRaw: string;
    name: string;
    kind: ComponentType;
    customData: unknown;

    constructor(from: Extract<Internal.ModalInteractionComponent, {customId: string}>) {
        [this.name, this.customData] = parseInteractionCustomId(from.customId);
        this.customIdRaw = from.customId;
        this.kind = from.kind;
    }
}

export class ModalSubmitInteractionValue extends ModalSubmitInteractionValueBase {
    value: string;

    constructor(from: Extract<Internal.ModalInteractionComponent, {value: string}>) {
        super(from);
        this.value = from.value;
    }
}

export class ModalSubmitInteractionValueSelectMenu extends ModalSubmitInteractionValueBase {
    kind: "SelectMenu" = "SelectMenu";

    values: string[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "SelectMenu"}>) {
        super(from);

        this.values = from.values;
    }
}

export class ModalSubmitInteractionValueChannelSelectMenu extends ModalSubmitInteractionValueBase {
    kind: "ChannelSelectMenu" = "ChannelSelectMenu";

    values: InteractionChannel[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "ChannelSelectMenu"}>, resolved: Internal.InteractionDataMap | null) {
        super(from);
        
        this.values = InteractionDataResolver.fromChannelSelectMenu(from.values, resolved);
    }
}

export class ModalSubmitInteractionValueRoleSelectMenu extends ModalSubmitInteractionValueBase {
    kind: "RoleSelectMenu" = "RoleSelectMenu";

    values: Role[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "RoleSelectMenu"}>, resolved: Internal.InteractionDataMap | null) {
        super(from);
        
        this.values = InteractionDataResolver.fromRoleSelectMenu(from.values, resolved);
    }
}

export class ModalSubmitInteractionValueUserSelectMenu extends ModalSubmitInteractionValueBase {
    kind: "UserSelectMenu" = "UserSelectMenu";

    values: InteractionUser[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "UserSelectMenu"}>, resolved: Internal.InteractionDataMap | null) {
        super(from);
        
        this.values = InteractionDataResolver.fromUserSelectMenu(from.values, resolved);
    }
}

export class ModalSubmitInteractionValueMentionableSelectMenu extends ModalSubmitInteractionValueBase {
    kind: "MentionableSelectMenu" = "MentionableSelectMenu"

    values: InteractionMentionable[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "MentionableSelectMenu"}>, resolved: Internal.InteractionDataMap | null) {
        super(from);
        
        this.values = InteractionDataResolver.fromMentionablesSelectMenu(from.values, resolved);
    }
}

export class ModalSubmitInteractionValueCheckbox extends ModalSubmitInteractionValueBase {
    kind: "Checkbox" = "Checkbox";

    checked: boolean;

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "Checkbox"}>) {
        super(from);

        this.checked = from.value;
    }
}

export class ModalSubmitInteractionValueCheckboxGroup extends ModalSubmitInteractionValueBase {
    kind: "CheckboxGroup" = "CheckboxGroup";

    values: string[];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "CheckboxGroup"}>) {
        super(from);

        this.values = from.values;
    }
}

export class ModalSubmitInteractionValueFileUpload extends ModalSubmitInteractionValueBase {
    kind: "FileUpload" = "FileUpload"

    values: Attachment[] = [];

    constructor(from: Extract<Internal.ModalInteractionComponent, {kind: "FileUpload"}>, resolved: Internal.InteractionDataMap | null) {
        super(from);
        
        this.values = InteractionDataResolver.fromAttachmentsUpload(from.values, resolved);
    }
}

export class SelectMenuInteraction extends ComponentInteraction {
    values: string[];

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = interaction.values;
    }
}

export interface InteractionChannel {
    id: string;
    kind: ChannelType;
    name: string;
    parentId?: string;
    permissionsRaw: string;
    threadMetadata?: ThreadMetadata;
}

export class ChannelSelectMenuInteraction extends ComponentInteraction {
    values: InteractionChannel[] = [];

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = InteractionDataResolver.fromChannelSelectMenu(interaction.values, interaction.resolved);
    }
}

export class RoleSelectMenuInteraction extends ComponentInteraction {
    values: Role[] = [];

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = InteractionDataResolver.fromRoleSelectMenu(interaction.values, interaction.resolved);
    }
}

export class UserSelectMenuInteraction extends ComponentInteraction {
    values: InteractionUser[] = [];

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = InteractionDataResolver.fromUserSelectMenu(interaction.values, interaction.resolved);
    }
}

export class MentionableSelectMenuInteraction extends ComponentInteraction {
    values: InteractionMentionable[] = [];

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = InteractionDataResolver.fromMentionablesSelectMenu(interaction.values, interaction.resolved);
    }
}

/**
 * Creates a 'customId' for you to use in message component's 'customId fields
 * 
 * This is needed as otherwise the interaction will not be handled by botloader.
 * 
 * DO NOT try to emulate this function yourself, we may change to different techniques entirely in the future and if you try to 
 * emulate this function them by implmenting it yourself you WILL break stuff.
 * 
 * Note that name + json(data) has to be less than 80 characters
 * 
 * @param name Name of the component, this is used when creating listeners using {@link Script.onInteractionButton} and {@link Script.onInteractionSelectMenu}
 * @param data Arbitrary data that will be passed to the interaction handlers, can be used to track a small amount of state.
 * Note that name + json(data) has to be less than 80 characters
 * @returns The customId for use in the customId field
 */
export function encodeInteractionCustomId(name: string, data: any) {
    let res = name + ":";
    if (data !== undefined && data !== null) {
        res += JSON.stringify(data);
    }

    // The string iterator that is used here iterates over characters,
    // not mere code units
    let length = [...res].length;
    if (res.length >= 95) {
        throw new Error("name + JSON.stringify(data) exceeds 95 characters")
    }

    return "0:" + res
}

export function parseInteractionCustomId(raw: string): [string, unknown] {
    let customId = raw.slice(2);
    let nameEnd = customId.indexOf(":");
    let name = "";
    let extras: unknown = null;

    if (nameEnd > -1) {
        name = customId.slice(0, nameEnd)
        let extrasStr = customId.slice(nameEnd + 1);
        if (extrasStr) {
            extras = JSON.parse(extrasStr);
        }
    }

    return [name, extras]
}

export class AutocompleteInteraction {
    member: Member;
    input: string;
    channelId: string;

    constructor(interaction: Internal.CommandInteraction, input: string) {
        this.member = new Member(interaction.member);
        this.channelId = interaction.channelId
        this.input = input
    }
}

export type InteractionType =
  | "Ping"
  | "ApplicationCommand"
  | "MessageComponent"
  | "ApplicationCommandAutocomplete"
  | "ModalSubmit";

export class InteractionMetadata {
    id: string;
    interactedMessageId: string | null;
    kind: InteractionType;
    originalResponseMessageId: string | null;
    targetMessageId: string | null;
    targetUser: User | null;
    triggeringInteractionMetadata: InteractionMetadata | null;
    user: User;

    constructor(interactionMetadata: Internal.IInteractionMetadata) {
        this.id = interactionMetadata.id;
        this.interactedMessageId = interactionMetadata.interactedMessageId;
        this.kind = interactionMetadata.kind;
        this.originalResponseMessageId = interactionMetadata.originalResponseMessageId;
        this.targetMessageId = interactionMetadata.targetMessageId;
        this.targetUser = interactionMetadata.targetUser ? new User(interactionMetadata.targetUser) : null;
        this.triggeringInteractionMetadata = interactionMetadata.triggeringInteractionMetadata ? new InteractionMetadata(interactionMetadata.triggeringInteractionMetadata) : null;
        this.user = new User(interactionMetadata.user);
    }
}