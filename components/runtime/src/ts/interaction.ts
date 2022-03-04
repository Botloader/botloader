import { ComponentType, CreateMessageFields, Member, toOpMessageFields } from "./discord";
import { Internal } from "./generated";
import { OpWrappers } from "./op_wrappers";

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

    async sendCallbackWithMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "ChannelMessageWithSource",
                fields: toOpMessageFields(fields),
                flags: flags || {},
            }
        })
    }
    async sendCallbackWithDeferredMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "DeferredChannelMessageWithSource",
                fields: toOpMessageFields(fields),
                flags: flags || {},
            }
        })
    }

    /**
     * @deprecated use {@link sendFollowup} instead
     */
    async sendResponse(resp: string | CreateMessageFields) {
        return this.sendFollowup(resp);
    }

    async sendFollowup(resp: string | CreateMessageFields) {
        if (typeof resp === "string") {
            await OpWrappers.createInteractionFollowup({
                interactionToken: this.token,
                fields: { content: resp }
            })
        } else {
            await OpWrappers.createInteractionFollowup({
                interactionToken: this.token,
                fields: toOpMessageFields(resp),
            })
        }
    }

    async deleteFollowup(id: string) {
        return OpWrappers.deleteInteractionFollowup(this.token, id);
    }

    async deleteOriginalResponse() {
        return OpWrappers.deleteInteractionOriginal(this.token);
    }
}

export class ComponentInteraction extends Interaction {
    customIdRaw: string;
    componentType: ComponentType;
    channelId: string;

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction.id, interaction.token, interaction.member);

        this.componentType = interaction.componentType;
        this.customIdRaw = interaction.customId;
        this.channelId = interaction.channelId;
    }


    async sendCallbackUpdateMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "UpdateMessage",
                fields: toOpMessageFields(fields),
                flags: flags || {},
            }
        })
    }

    async sendCallbackDeferredUpdateMessage() {
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

export interface InteractionMessageFlags {
    ephemeral?: boolean,
}

export function encodeInteractionCustomId(name: string, data: any) {
    let res = "0:" + name + ":";
    if (data !== undefined && data !== null) {
        res += JSON.stringify(data);
    }

    return res
}