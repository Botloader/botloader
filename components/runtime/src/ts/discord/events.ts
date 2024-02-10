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
import type { IEventInviteCreate } from "../generated/internal/IEventInviteCreate";
import type { IEventInviteDelete } from "../generated/internal/IEventInviteDelete";
import type { IInviteTargetUser } from "../generated/discord/IInviteTargetUser";
import type { InviteTargetType } from "../generated/discord/InviteTargetType";
import type { IEventVoiceStateUpdate } from "../generated/internal/IEventVoiceStateUpdate";
import type { IVoiceState } from "../generated/internal/IVoiceState";

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

export class EventInviteCreate {
    channelId: string;
    code: string;
    createdAt: number;
    inviter?: User;
    maxAge: number;
    maxUses: number;
    targetUserType?: InviteTargetType;
    targetUser?: IInviteTargetUser;
    temporary: boolean;
    uses: number;

    /** 
    * @internal 
    */
    constructor(json: IEventInviteCreate) {
        this.channelId = json.channelId
        this.code = json.code
        this.createdAt = json.createdAt
        this.inviter = json.inviter && new User(json.inviter)
        this.maxAge = json.maxAge
        this.maxUses = json.maxUses
        this.targetUserType = json.targetUserType
        this.targetUser = json.targetUser
        this.temporary = json.temporary
        this.uses = json.uses
    }
}



export class EventInviteDelete {
    channelId: string;
    code: string;


    /** 
    * @internal 
    */
    constructor(json: IEventInviteDelete) {
        this.channelId = json.channelId
        this.code = json.code
    }
}

export class VoiceState {
    channelId: string | null;
    deaf: boolean;
    member?: Member;
    mute: boolean;
    selfDeaf: boolean;
    selfMute: boolean;
    selfStream: boolean;
    selfVideo: boolean;
    sessionId: string;
    suppress: boolean;
    userId: string;
    requestToSpeakTimestamp: number | null;

    /** 
    * @internal 
    */
    constructor(json: IVoiceState) {
        this.channelId = json.channelId
        this.deaf = json.deaf
        this.mute = json.mute
        this.selfDeaf = json.selfDeaf
        this.selfMute = json.selfMute
        this.selfStream = json.selfStream
        this.selfVideo = json.selfVideo
        this.sessionId = json.sessionId
        this.suppress = json.suppress
        this.userId = json.userId
        this.requestToSpeakTimestamp = json.requestToSpeakTimestamp

        if (json.member) {
            this.member = new Member(json.member)
        }
    }
}

export class EventVoiceStateUpdate extends VoiceState {
    oldState?: VoiceState

    /** 
    * @internal 
    */
    constructor(json: IEventVoiceStateUpdate) {
        super(json.new)

        if (json.old) {
            this.oldState = new VoiceState(json.old)
            this.oldState.member = this.member
        }
    }
}