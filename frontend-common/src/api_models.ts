export interface User {
    avatar?: string,
    bot: boolean,
    discriminator: string,
    email?: string,
    flags?: number,
    id: string,
    locale?: string,
    username: string,
    premium_type?: number,
    public_flags?: number,
    verified?: boolean,
}


export interface UserGuild {
    id: string,
    name: string,
    icon?: string,
    owner: boolean,
    permissions: string,
    features: string[],
}

export interface BotGuild {
    guild: UserGuild,
    connected: boolean,
}

export interface CurrentGuildsResponse {
    guilds: BotGuild[],
}

export interface LoginResponse {
    user: User,
    token: string,
}

export interface SessionMeta {
    kind: SessionType,
    created_at: string,
    token: string,
}

export type SessionType = "User" | "ApiKey";


export interface Script {
    id: number,
    name: string,
    original_source: string,
    compiled_js: string,
    enabled: boolean,
}

export interface CreateScript {
    name: string,
    original_source: string,
    enabled: boolean,
}

export interface UpdateScript {
    name?: string,
    original_source?: string,
    enabled?: boolean,
}

export interface EmptyResponse { }


export interface GuildMetaConfig {
    guild_id: string,
    error_channel_id: string | null,
}