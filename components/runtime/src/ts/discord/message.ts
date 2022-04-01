import { User } from "./user";
import { getCurrentGuildId } from "./dapi";

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
import type { IUserMention } from "../generated/internal/UserMention";

export class Message {
    activity: MessageActivity | null;
    application: MessageApplication | null;
    attachments: Attachment[];
    author: User;
    channelId: string;
    content: string;
    components: IComponent[];
    editedTimestamp: number | null;
    embeds: Embed[];
    flags: MessageFlags | null;
    guildId: string;
    id: string;
    kind: MessageType;
    member: PartialMember | null;
    mentionChannels: ChannelMention[];
    mentionEveryone: boolean;
    mentionRoles: string[];
    mentions: UserMention[];
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
        this.content = json.content;
        this.components = json.components;
        this.editedTimestamp = json.editedTimestamp;
        this.embeds = json.embeds;
        this.flags = json.flags;
        this.guildId = json.guildId ?? getCurrentGuildId();
        this.id = json.id;
        this.kind = json.kind;
        this.member = json.member;
        this.mentionChannels = json.mentionChannels;
        this.mentionEveryone = json.mentionEveryone;
        this.mentionRoles = json.mentionRoles;
        this.mentions = json.mentions.map(v => new UserMention(v));
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
}

export class UserMention extends User {
    member: PartialMember | null;

    /**
     * @internal
     */
    constructor(json: IUserMention) {
        super(json.user);

        this.member = json.member;
    }
}