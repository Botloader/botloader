import { Member } from "./member";
import { User } from "./user";
import { UserMention } from "./message";

import type { ReactionType } from "../generated/discord/ReactionType";
import type { Attachment } from "../generated/discord/Attachment";
import type { Embed } from "../generated/discord/Embed";
import type { MessageType } from "../generated/discord/MessageType";

import type { IEventMessageUpdate } from "../generated/internal/EventMessageUpdate";
import type { IEventMessageReactionAdd } from "../generated/internal/EventMessageReactionAdd";
import type { IEventMemberRemove } from "../generated/internal/EventMemberRemove";

export class EventMessageReactionAdd {
    channelId: string;
    messageId: string;
    emoji: ReactionType;
    member: Member;
    userId: string;

    /** 
     * @internal 
     */
    constructor(json: IEventMessageReactionAdd) {
        this.channelId = json.channelId;
        this.messageId = json.messageId;
        this.emoji = json.emoji;
        this.member = new Member(json.member);
        this.userId = json.userId;
    }
}

export class EventMessageUpdate {
    attachments?: Attachment[];
    author?: User
    channelId: string;
    content: string | null;
    editedTimestamp?: number;
    embeds?: Embed[];
    guildId?: string;
    id: string;
    kind?: MessageType;
    mentionEveryone?: boolean;
    mentionRoles?: string[];
    mentions?: UserMention[];
    pinned?: boolean;
    timestamp?: number;
    tts?: boolean;


    /** 
     * @internal 
     */
    constructor(json: IEventMessageUpdate) {
        this.attachments = json.attachments;
        this.author = json.author ? new User(json.author) : undefined;
        this.channelId = json.channelId;
        this.content = json.content;
        this.editedTimestamp = json.editedTimestamp;
        this.embeds = json.embeds;
        this.guildId = json.guildId;
        this.id = json.id;
        this.kind = json.kind;
        this.mentionEveryone = json.mentionEveryone;
        this.mentionRoles = json.mentionRoles;
        this.mentions = json.mentions?.map(v => new UserMention(v));
        this.pinned = json.pinned;
        this.timestamp = json.timestamp;
        this.tts = json.tts;
    }
}


export class EventMemberRemove {
    guildId: string;
    user: User;

    /** 
     * @internal 
     */
    constructor(json: IEventMemberRemove) {
        this.guildId = json.guildId;
        this.user = new User(json.user);
    }
}