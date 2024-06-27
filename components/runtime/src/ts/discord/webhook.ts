import { ExractClassProperties } from "../core_util";
import { User, UserFields } from "./user";
import type * as Internal from "../generated/internal/index"
import { EditWebhookFields, deleteWebhook, editWebhook, editWebhookWithToken, executeWebhook, getCurrentGuildId, type CreateMessageFields } from "./dapi";

export type WebhookFields = ExractClassProperties<Omit<Webhook, "user">> & { user: UserFields | null }

export class Webhook {
    id: string;
    applicationId: string | null;
    avatar: string | null;
    channelId: string;
    guildId: string | null;
    kind: WebhookType;
    name: string | null;
    sourceChannel: WebhookChannel | null;
    sourceGuild: WebhookGuild | null;

    /**
     * Only present for the webhook type of 'Incoming'
     */
    token: string | null;

    url: string | null;
    user: User | null;

    constructor(fields: WebhookFields) {
        this.id = fields.id
        this.applicationId = fields.applicationId
        this.avatar = fields.avatar
        this.channelId = fields.channelId
        this.guildId = fields.guildId
        this.kind = fields.kind
        this.name = fields.name
        this.sourceChannel = fields.sourceChannel
        this.sourceGuild = fields.sourceGuild
        this.token = fields.token
        this.url = fields.url

        this.user = fields.user && new User(fields.user)
    }

    /**
     * @internal
     */
    static fromInternal(w: Internal.DiscordWebhook) {
        return new Webhook(w)
    }

    edit(fields: EditWebhookFields) {
        const currentGuildId = getCurrentGuildId()
        if (this.guildId !== currentGuildId) {
            if (!this.token) {
                throw new Error("Can't edit webhooks for other guilds if no token is provided")
            }

            return editWebhookWithToken(this.id, this.token, fields)
        } else {
            return editWebhook(this.id, fields)
        }
    }

    delete() {
        const currentGuildId = getCurrentGuildId()
        if (this.guildId !== currentGuildId) {
            if (!this.token) {
                throw new Error("Can't delete webhooks for other guilds if no token is provided")
            }

            return deleteWebhook(this.id, this.token)
        } else {
            return deleteWebhook(this.id)
        }
    }

    execute(fields: CreateMessageFields) {
        if (!this.token) {
            throw new Error("Webhook token is null")
        }

        return executeWebhook(this.id, this.token, fields)
    }
}

export type WebhookType =
    | "Incoming"
    | "ChannelFollower"
    | "Application"
    | "Unknown";

export interface WebhookChannel {
    id: string;
    name: string;
}

export interface WebhookGuild {
    icon: string | null;
    id: string;
    name: string;
}
