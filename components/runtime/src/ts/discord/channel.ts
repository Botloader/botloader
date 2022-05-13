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
import { getPins } from "./dapi";

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
    } else if (json.kind === "Text" || json.kind === "News" || json.kind === "Store" || json.kind === "Forum" || json.kind === "GuildDirectory") {
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

    throw new Error("unknown channel type: " + json.kind)
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

    isTextChannel(): this is TextChannel {
        return this instanceof TextChannel;
    }

    isVoiceChannel(): this is VoiceChannel {
        return this instanceof VoiceChannel;
    }

    isAnyThread(): this is (NewsThread | PrivateThread | PublicThread) {
        return this.isNewsThread() || this.isPrivateThread() || this.isPublicThread();
    }

    pins() {
        return getPins(this.id);
    }
}

export class PrivateThread extends BaseChannel {
    kind: "PrivateThread" = "PrivateThread";
    defaultAutoArchiveDurationMinutes: number | null;
    invitable: boolean | null;
    member: SelfThreadMember | null;
    memberCount: number;
    messageCount: number;
    ownerId: string | null;
    parentId: string | null;
    permissionOverwrites: IPermissionOverwrite[];
    rateLimitPerUser: number | null;
    threadMetadata: ThreadMetadata;

    /**
     * @internal
     */
    constructor(json: IPrivateThread) {
        super(json);

        this.defaultAutoArchiveDurationMinutes = json.defaultAutoArchiveDurationMinutes;
        this.invitable = json.invitable;
        this.member = json.member ? new SelfThreadMember(json.member) : null;
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
    member: SelfThreadMember | null;
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
        this.member = json.member ? new SelfThreadMember(json.member) : null;
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
    permissionOverwrites: IPermissionOverwrite[];
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
    kind: "Text" | "News" | "Store" | "Forum" | "GuildDirectory";
    lastPinTimestamp: number | null;
    nsfw: boolean;
    parentId: string | null;
    permissionOverwrites: IPermissionOverwrite[];
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
    member: SelfThreadMember | null;
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
        this.member = json.member ? new SelfThreadMember(json.member) : null;
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
    permissionOverwrites: IPermissionOverwrite[];
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
