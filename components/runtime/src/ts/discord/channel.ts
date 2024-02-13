import { Member } from "./member";

import type { IPermissionOverwrite } from "../generated/discord/IPermissionOverwrite";
import type { ThreadMetadata } from "../generated/discord/ThreadMetadata";
import type { VideoQualityMode } from "../generated/discord/VideoQualityMode";
import type { ChannelType } from "../generated/discord/ChannelType";

import type { InternalGuildChannel } from "../generated/internal/GuildChannel";
import type { IPrivateThread } from "../generated/internal/PrivateThread";
import type { IThreadMember } from "../generated/internal/ThreadMember";
import type { IPublicThread } from "../generated/internal/PublicThread";
import type { ICategoryChannel } from "../generated/internal/CategoryChannel";
import type { INewsThread } from "../generated/internal/NewsThread";
import type { ITextChannel } from "../generated/internal/TextChannel";
import type { IVoiceChannel } from "../generated/internal/VoiceChannel";
import type { ISelfThreadMember } from "../generated/internal/ISelfThreadMember";
import { ICreateForumThread, ICreateStandaloneThread, ICreateThreadFromMessage, IEditThread, createForumThread, createStandaloneThread, createThreadFromMessage, editChannel, editThread, getPins } from "./dapi";

export type GuildChannel =
    | CategoryChannel
    | NewsThread
    | PrivateThread
    | PublicThread
    | TextChannel
    | VoiceChannel
    | UnknownChannel;


/**
 * @internal
 */
export function guildChannelFromInternal(json: InternalGuildChannel): GuildChannel {
    if (json.kind === "Voice" || json.kind === "StageVoice") {
        return new VoiceChannel(json);
    } else if (json.kind === "Text" || json.kind === "News" || json.kind === "Forum" || json.kind === "GuildDirectory") {
        return new TextChannel(json);
    } else if (json.kind === "Category") {
        return new CategoryChannel(json);
    } else if (json.kind === "NewsThread") {
        return new NewsThread(json);
    } else if (json.kind === "PrivateThread") {
        return new PrivateThread(json);
    } else if (json.kind === "PublicThread") {
        return new PublicThread(json);
    } else {
        return new UnknownChannel(json);
    }
}

export function threadChannelFromInternal(json: InternalGuildChannel): Thread {
    const channel = guildChannelFromInternal(json)
    if (channel.isThread()) {
        return channel
    } else {
        throw new Error(`Channel is not thread: id: ${json.id}, kind: ${json.kind}`)
    }
}

export abstract class BaseChannel {
    id: string;
    kind: ChannelType;
    name: string;
    permissionOverwrites: IPermissionOverwrite[];

    /**
     * @internal
     */
    constructor(json: InternalGuildChannel) {
        this.id = json.id;
        this.kind = json.kind;
        if ('name' in json) {
            this.name = json.name;
        } else {
            this.name = ""
        }

        if ('permissionOverwrites' in json) {
            this.permissionOverwrites = json.permissionOverwrites
        } else {
            this.permissionOverwrites = []
        }

    }

    isCategoryChannel(): this is CategoryChannel {
        return this instanceof CategoryChannel;
    }

    isNewsThread(): this is NewsThread {
        return this instanceof NewsThread;
    }

    isPrivateThread(): this is PrivateThread {
        return this instanceof PrivateThread;
    }

    isPublicThread(): this is PublicThread {
        return this instanceof PublicThread;
    }

    isThread(): this is Thread {
        return this instanceof Thread;
    }

    isTextChannel(): this is TextChannel {
        return this instanceof TextChannel;
    }

    isVoiceChannel(): this is VoiceChannel {
        return this instanceof VoiceChannel;
    }

    isAnyThread(): this is (NewsThread | PrivateThread | PublicThread) {
        return this.isThread()
    }

    pins() {
        return getPins(this.id);
    }
}

export class UnknownChannel extends BaseChannel { }

export class CategoryChannel extends BaseChannel {
    kind: "Category" = "Category";
    position: number;

    /**
     * @internal
     */
    constructor(json: ICategoryChannel) {
        super(json);

        this.position = json.position;
    }
}

export class TextChannel extends BaseChannel {
    kind: "Text" | "News" | "Forum" | "GuildDirectory";
    lastPinTimestamp: number | null;
    nsfw: boolean;
    parentId: string | null;
    position: number;
    rateLimitPerUser: number | null;
    topic: string | null;

    /**
     * @internal
     */
    constructor(json: ITextChannel) {
        super(json);

        this.kind = json.kind;
        this.lastPinTimestamp = json.lastPinTimestamp;
        this.nsfw = json.nsfw;
        this.parentId = json.parentId;
        this.position = json.position;
        this.rateLimitPerUser = json.rateLimitPerUser;
        this.topic = json.topic;
    }

    createForumThread(fields: Omit<ICreateForumThread, "channelId">) {
        if (this.kind !== "Forum") {
            throw new Error(`This channel is not a forum: ${this.id}`)
        }

        return createForumThread({
            ...fields,
            channelId: this.id,
        })
    }

    createStandaloneThread(fields: Omit<ICreateStandaloneThread, "channelId">) {
        if (this.kind === "Forum") {
            throw new Error(`This channel is a forum: ${this.id}`)
        }

        return createStandaloneThread({
            ...fields,
            channelId: this.id,
        })
    }

    createThreadFromMessage(fields: Omit<ICreateThreadFromMessage, "channelId">) {
        if (this.kind === "Forum") {
            throw new Error(`This channel is a forum: ${this.id}`)
        }

        return createThreadFromMessage({
            ...fields,
            channelId: this.id
        })
    }
}

export abstract class Thread extends BaseChannel {
    defaultAutoArchiveDurationMinutes: number | null;
    kind: "NewsThread" | "PrivateThread" | "PublicThread";
    member: SelfThreadMember | null;
    memberCount: number;
    messageCount: number;
    ownerId: string | null;
    parentId: string | null;
    rateLimitPerUser: number | null;
    threadMetadata: ThreadMetadata;

    constructor(json: IPrivateThread | INewsThread | IPublicThread) {
        super(json)

        this.defaultAutoArchiveDurationMinutes = json.defaultAutoArchiveDurationMinutes
        this.kind = json.kind
        this.member = json.member
        this.memberCount = json.memberCount
        this.messageCount = json.messageCount
        this.ownerId = json.ownerId
        this.parentId = json.parentId
        this.rateLimitPerUser = json.rateLimitPerUser
        this.threadMetadata = json.threadMetadata
    }

    async edit(fields: Omit<IEditThread, "channelId">): Promise<Thread> {
        return await editThread({
            ...fields,
            channelId: this.id,
        })
    }

    archive() {
        return this.edit({
            archived: true,
        })
    }

    unArchive() {
        return this.edit({
            archived: false,
        })
    }

    lock() {
        return this.edit({
            locked: true,
        })
    }

    unlock() {
        return this.edit({
            locked: false,
        })
    }
}


export class PrivateThread extends Thread {
    kind: "PrivateThread" = "PrivateThread";
    invitable: boolean | null;

    /**
     * @internal
     */
    constructor(json: IPrivateThread) {
        super(json);

        this.invitable = json.invitable;
    }
}

export class PublicThread extends Thread {
    kind: "PublicThread" = "PublicThread";

    /**
     * @internal
     */
    constructor(json: IPublicThread) {
        super(json);
    }
}

export class NewsThread extends BaseChannel {
    kind: "NewsThread" = "NewsThread";

    /**
     * @internal
     */
    constructor(json: INewsThread) {
        super(json);
    }
}

export class VoiceChannel extends BaseChannel {
    bitrate: number;
    kind: "Voice" | "StageVoice";
    parentId: string | null;
    position: number;
    rtcRegion: string | null;
    userLimit: number | null;
    videoQualityMode: VideoQualityMode | null;

    /**
     * @internal
     */
    constructor(json: IVoiceChannel) {
        super(json);

        this.bitrate = json.bitrate;
        this.kind = json.kind;
        this.parentId = json.parentId;
        this.position = json.position;
        this.rtcRegion = json.rtcRegion;
        this.userLimit = json.userLimit;
        this.videoQualityMode = json.videoQualityMode;
    }
}

export class SelfThreadMember {
    /**
     * When this use joined the thread, in unix milliseconds time
     */
    joinTimestamp: number;

    /**
     * @internal
     */
    constructor(json: ISelfThreadMember) {
        this.joinTimestamp = json.joinTimestamp;
    }
}

export class ThreadMember extends SelfThreadMember {
    id: string | null;
    member: Member | null;
    userId: string | null;

    /**
     * @internal
     */
    constructor(json: IThreadMember) {
        super(json)
        this.id = json.id;
        this.member = json.member ? new Member(json.member) : null;
        this.userId = json.userId;
    }
}
