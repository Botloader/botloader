import { Member } from "./member";

import type { PermissionOverwrite } from "../generated/discord/PermissionOverwrite";
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

export type GuildChannel =
    | CategoryChannel
    | NewsThread
    | PrivateThread
    | PublicThread
    | TextChannel
    | VoiceChannel;


/**
 * @internal
 */
export function guildChannelFromInternal(json: InternalGuildChannel): GuildChannel {
    if (json.kind === "Voice" || json.kind === "StageVoice") {
        return new VoiceChannel(json);
    } else if (json.kind === "Text" || json.kind === "News" || json.kind === "Store") {
        return new TextChannel(json);
    } else if (json.kind === "Category") {
        return new CategoryChannel(json);
    } else if (json.kind === "NewsThread") {
        return new NewsThread(json);
    } else if (json.kind === "PrivateThread") {
        return new PrivateThread(json);
    } else if (json.kind === "PublicThread") {
        return new PublicThread(json);
    }

    throw new Error("unknown channel")
}

export abstract class BaseChannel {
    id: string;
    kind: ChannelType;
    name: string;


    /**
     * @internal
     */
    constructor(json: InternalGuildChannel) {
        this.id = json.id;
        this.kind = json.kind;
        this.name = json.name;
    }
}

export class PrivateThread extends BaseChannel {
    kind: "PrivateThread" = "PrivateThread";
    defaultAutoArchiveDurationMinutes: number | null;
    invitable: boolean | null;
    member: ThreadMember | null;
    memberCount: number;
    messageCount: number;
    ownerId: string | null;
    parentId: string | null;
    permissionOverwrites: PermissionOverwrite[];
    rateLimitPerUser: number | null;
    threadMetadata: ThreadMetadata;

    /**
     * @internal
     */
    constructor(json: IPrivateThread) {
        super(json);

        this.defaultAutoArchiveDurationMinutes = json.defaultAutoArchiveDurationMinutes;
        this.invitable = json.invitable;
        this.member = json.member ? new ThreadMember(json.member) : null;
        this.memberCount = json.memberCount;
        this.messageCount = json.messageCount;
        this.ownerId = json.ownerId;
        this.parentId = json.parentId;
        this.permissionOverwrites = json.permissionOverwrites;
        this.rateLimitPerUser = json.rateLimitPerUser;
        this.threadMetadata = json.threadMetadata;
    }
}

export class PublicThread extends BaseChannel {
    kind: "PublicThread" = "PublicThread";
    defaultAutoArchiveDurationMinutes: number | null;
    member: ThreadMember | null;
    memberCount: number;
    messageCount: number;
    ownerId: string | null;
    parentId: string | null;
    rateLimitPerUser: number | null;
    threadMetadata: ThreadMetadata;

    /**
     * @internal
     */
    constructor(json: IPublicThread) {
        super(json);

        this.defaultAutoArchiveDurationMinutes = json.defaultAutoArchiveDurationMinutes;
        this.member = json.member ? new ThreadMember(json.member) : null;
        this.memberCount = json.memberCount;
        this.messageCount = json.messageCount;
        this.ownerId = json.ownerId;
        this.parentId = json.parentId;
        this.rateLimitPerUser = json.rateLimitPerUser;
        this.threadMetadata = json.threadMetadata;
    }
}

export class CategoryChannel extends BaseChannel {
    kind: "Category" = "Category";
    permissionOverwrites: PermissionOverwrite[];
    position: bigint;

    /**
     * @internal
     */
    constructor(json: ICategoryChannel) {
        super(json);

        this.permissionOverwrites = json.permissionOverwrites;
        this.position = json.position;
    }
}

export class TextChannel extends BaseChannel {
    kind: "Text" | "News" | "Store";
    lastPinTimestamp: number | null;
    nsfw: boolean;
    parentId: string | null;
    permissionOverwrites: PermissionOverwrite[];
    position: bigint;
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
        this.permissionOverwrites = json.permissionOverwrites;
        this.position = json.position;
        this.rateLimitPerUser = json.rateLimitPerUser;
        this.topic = json.topic;
    }
}

export class NewsThread extends BaseChannel {
    defaultAutoArchiveDurationMinutes: number | null;
    kind: "NewsThread" = "NewsThread";
    member: ThreadMember | null;
    memberCount: number;
    messageCount: number;
    ownerId: string | null;
    parentId: string | null;
    rateLimitPerUser: number | null;
    threadMetadata: ThreadMetadata;

    /**
     * @internal
     */
    constructor(json: INewsThread) {
        super(json);

        this.defaultAutoArchiveDurationMinutes = json.defaultAutoArchiveDurationMinutes;
        this.kind = json.kind;
        this.member = json.member ? new ThreadMember(json.member) : null;
        this.memberCount = json.memberCount;
        this.messageCount = json.messageCount;
        this.ownerId = json.ownerId;
        this.parentId = json.parentId;
        this.rateLimitPerUser = json.rateLimitPerUser;
        this.threadMetadata = json.threadMetadata;
    }

}
export class VoiceChannel extends BaseChannel {
    bitrate: number;
    kind: "Voice" | "StageVoice";
    parentId: string | null;
    permissionOverwrites: PermissionOverwrite[];
    position: bigint;
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
        this.permissionOverwrites = json.permissionOverwrites;
        this.position = json.position;
        this.rtcRegion = json.rtcRegion;
        this.userLimit = json.userLimit;
        this.videoQualityMode = json.videoQualityMode;
    }
}


export class ThreadMember {
    id: string | null;
    joinTimestamp: number;
    member: Member | null;
    userId: string | null;

    /**
     * @internal
     */
    constructor(json: IThreadMember) {
        this.id = json.id;
        this.joinTimestamp = json.joinTimestamp;
        this.member = json.member ? new Member(json.member) : null;
        this.userId = json.userId;
    }
}