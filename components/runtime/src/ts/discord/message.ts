import { User, UserMention } from "./user";
import { InteractionMetadata } from "./interaction";
import { MessageSnapshot } from "./snapshot";
import { CreateMessageFields, ICreateThreadFromMessage, createMessage, createPin, createThreadFromMessage, deleteMessage, deletePin, editMessage, getCurrentGuildId } from "./dapi";

import type { Attachment } from "../generated/discord/Attachment";
import type { ChannelMention } from "../generated/discord/ChannelMention";
import type { IComponent } from "../generated/discord/IComponent";
import type { Embed } from "../generated/discord/Embed";
import type { MessageActivity } from "../generated/discord/MessageActivity";
import type { MessageApplication } from "../generated/discord/MessageApplication";
import type { MessageFlags } from "../generated/discord/MessageFlags";
import type { MessageReaction } from "../generated/discord/MessageReaction";
import type { MessageReference } from "../generated/discord/MessageReference";
import type { MessageType } from "../generated/discord/MessageType";
import type { PartialMember } from "../generated/discord/PartialMember";

import type { IMessage } from "../generated/internal/IMessage";

export class Message {
    activity: MessageActivity | null;
    application: MessageApplication | null;
    attachments: Attachment[];
    author: User;
    channelId: string;
    components: IComponent[];
    content: string;
    editedTimestamp: number | null;
    embeds: Embed[];
    flags: MessageFlags | null;
    guildId: string;
    id: string;
    interactionMetadata: InteractionMetadata | null;
    kind: MessageType;
    member: PartialMember | null;
    mentionChannels: ChannelMention[];
    mentionEveryone: boolean;
    mentionRoles: string[];
    mentions: UserMention[];
    messageSnapshots: MessageSnapshot[];
    pinned: boolean;
    reactions: MessageReaction[];
    reference: MessageReference | null;
    referencedMessage: Message | null;
    timestamp: number;
    tts: boolean;
    webhookId: string | null;

    /**
     * @internal
     */
    constructor(json: IMessage) {
        this.activity = json.activity;
        this.application = json.application;
        this.attachments = json.attachments;
        this.author = new User(json.author);
        this.channelId = json.channelId;
        this.components = json.components;
        this.content = json.content;
        this.editedTimestamp = json.editedTimestamp;
        this.embeds = json.embeds;
        this.flags = json.flags;
        this.guildId = json.guildId ?? getCurrentGuildId();
        this.id = json.id;
        this.interactionMetadata = json.interactionMetadata ? new InteractionMetadata(json.interactionMetadata) : null;
        this.kind = json.kind;
        this.member = json.member;
        this.mentionChannels = json.mentionChannels;
        this.mentionEveryone = json.mentionEveryone;
        this.mentionRoles = json.mentionRoles;
        this.mentions = json.mentions.map(v => new UserMention(v));
        this.messageSnapshots = json.messageSnapshots.map(v => new MessageSnapshot(v));
        this.pinned = json.pinned;
        this.reactions = json.reactions;
        this.reference = json.reference;
        this.referencedMessage = json.referencedMessage ? new Message(json.referencedMessage) : null;
        this.timestamp = json.timestamp;
        this.tts = json.tts;
        this.webhookId = json.webhookId;


    }

    hyperlink() {
        return `https://discord.com/channels/${this.guildId}/${this.channelId}/${this.id}`
    }

    pin() {
        return createPin(this.channelId, this.id);
    }

    unPin() {
        return deletePin(this.channelId, this.id);
    }

    delete() {
        return deleteMessage(this.channelId, this.id);
    }

    edit(fields: CreateMessageFields) {
        return editMessage(this.channelId, this.id, fields);
    }

    forward(channelId: string) {
        return createMessage(channelId, {
            forward: {
                channelId: this.channelId,
                messageId: this.id
            }
        });
    }

    createThread(fields: Omit<ICreateThreadFromMessage, "messageId" | "channelId">) {
        return createThreadFromMessage({
            ...fields,
            channelId: this.channelId,
            messageId: this.id,
        })
    }
}
