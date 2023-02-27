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
    plugin_id: number | null,
    plugin_auto_update: boolean | null,
    plugin_version_number: number | null,
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

export interface Plugin<Variant = ScriptPluginData> {
    id: number,
    created_at: string,
    author_id: string,
    name: string,
    short_description: string,
    long_description: string,
    is_public: boolean,
    is_official: boolean,
    data: Variant,
    current_version: number,
}

export interface ScriptPluginData {
    plugin_type: "ScriptPlugin",
    published_version: string | null,
    published_version_updated_at: string | null,
    dev_version: string | null,
    dev_version_updated_at: string | null,
}

export type ScriptPlugin = Plugin<ScriptPluginData>;

export interface ScriptsWithPlugins {
    scripts: Script[],
    plugins: Plugin[]
}