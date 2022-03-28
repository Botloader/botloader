import { ComponentType } from '../generated/discord/index';
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

    get hasSentCallback() {
        return this._hasSentCallback;
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
     * @deprecated use {@link ackWithMessage} 
     */
    async sendCallbackWithMessage(fields: Internal.OpCreateMessageFields, flags?: InteractionMessageFlags) {
        this.ackWithMessage({
            ...fields,
            flags: flags,
        })
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
            ineractionToken: this.token,
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

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
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
        return editInteractionOriginalResponse(this.token, fields)
    }

    async deleteOriginalResponse() {
        return deleteInteractionOriginalResponse(this.token);
    }

    async getFollowup(messageId: string) {
        return getInteractionFollowupMessage(this.token, messageId);
    }

    /**
     * @deprecated use {@link createFollowup} instead
     */
    async sendFollowup(resp: string | InteractionCreateMessageFields) {
        return createInteractionFollowupMessage(this.token, resp);
    }

    async createFollowup(resp: string | InteractionCreateMessageFields) {
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
            ineractionToken: this.token,
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
            ineractionToken: this.token,
            data: {
                kind: "DeferredUpdateMessage",
            }
        })
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
    if (res.length >= 80) {
        throw new Error("name + JSON.stringify(data) exceeds 80 characters")
    }

    return "0:" + res
}