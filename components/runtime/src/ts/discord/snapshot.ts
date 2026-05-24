import { getCurrentGuildId } from "./dapi";
import { UserMention } from "./user";

import { Attachment } from "../generated/discord/Attachment";
import { IComponent } from "../generated/discord/IComponent";
import type { Embed } from "../generated/discord/Embed";
import type { MessageFlags } from "../generated/discord/MessageFlags";
import type { MessageType } from "../generated/discord/MessageType";

import type { IMessageSnapshot } from "../generated/internal/IMessageSnapshot";
import type { IMessageSnapshotFields } from "../generated/internal/IMessageSnapshotFields";

export class MessageSnapshot {
    message: MessageSnapshotFields;
    guildId?: string;

    /**
     * @internal
     */
    constructor(json: IMessageSnapshot) {
        this.message = new MessageSnapshotFields(json.message);
        this.guildId = json.guildId ?? getCurrentGuildId();
    }
}

export class MessageSnapshotFields {
    attachments: Attachment[];
    components: IComponent[];
    content: string;
    editedTimestamp: number | null;
    embeds: Embed[];
    flags?: MessageFlags;
    kind: MessageType;
    mentions: UserMention[];
    mentionRoles: string[];
    timestamp: number;


    /**
     * @internal
     */
    constructor(json: IMessageSnapshotFields) {
        this.attachments = json.attachments;
        this.components = json.components;
        this.content = json.content;
        this.editedTimestamp = json.editedTimestamp;
        this.embeds = json.embeds;
        this.flags = json.flags;
        this.kind = json.kind;
        this.mentionRoles = json.mentionRoles;
        this.mentions = json.mentions.map(v => new UserMention(v));
        this.timestamp = json.timestamp;
    }
}