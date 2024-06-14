import { ComponentType, IComponent } from '../generated/discord/index';
import * as Internal from '../generated/internal/index';
import { CreateMessageFields, InteractionMessageFlags, InteractionCreateMessageFields, createInteractionFollowupMessage, getInteractionFollowupMessage, deleteInteractionOriginalResponse, editInteractionOriginalResponse, getInteractionOriginalResponse, editInteractionFollowupMessage, deleteInteractionFollowupMessage, toOpMessageFields } from './dapi';
import { OpWrappers } from '../op_wrappers';
import { Member } from './member';
import { Message } from './message';

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
            const parsed = new ModalSubmitInteractionValue(elem);
            this.values = {
                ...this.values,
                [parsed.name]: parsed,
            }
        }
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

export interface ModalSubmitInteractionValues {
    [key: string]: ModalSubmitInteractionValue
}

export class ModalSubmitInteractionValue {
    customIdRaw: string;
    name: string;
    value: string;
    kind: ComponentType;

    customData: unknown;

    constructor(from: Internal.IModalInteractionDataComponent) {
        [this.name, this.customData] = parseInteractionCustomId(from.customId);
        this.customIdRaw = from.customId;
        this.kind = from.kind;
        this.value = from.value;
    }
}

export class SelectMenuInteraction extends ComponentInteraction {
    values: string[];


    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = interaction.values;
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