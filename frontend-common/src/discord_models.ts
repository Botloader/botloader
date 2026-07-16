// The spec exposes FullGuild.guild/channels/roles as untyped json (deep
// twilight structures, see webapi's discord_schemas.rs for the typed mirrors
// that do exist). These hand-written types cover the fields the frontend uses
// and override the generated FullGuild in index.ts.
import { FullGuild as GeneratedFullGuild } from "./generated/api";

export type FullGuild = Omit<GeneratedFullGuild, "guild" | "channels" | "roles"> & {
    guild: Guild,
    channels: DiscordChannel[],
    roles: DiscordRole[],
};

export interface Guild {
    icon: string | null
    id: string
}

export interface DiscordChannel {
    id: string,
    guild_id: string,
    icon?: string,
    invitable?: boolean,
    type: DiscordNumberedChannelTypes,
    name?: string,
    nsfw?: boolean,
    parent_id?: string,
    position?: number,
}

export enum DiscordNumberedChannelTypes {
    GuildText = 0,
    Private = 1,
    GuildVoice = 2,
    Group = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
}

export interface DiscordRole {
    id: string,
    color: number,
    hoist: boolean,
    icon?: string,
    managed: boolean,
    mentionable: boolean,
    name: string,
    permissions: string,
    position: number,
}
