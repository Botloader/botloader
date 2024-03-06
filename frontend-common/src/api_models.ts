export interface User {
    avatar: string | null,
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
    settings_definitions: SettingsOptionDefinition[] | null,
    settings_values: SettingsOptionValue[],
}

export type SettingsOptionDefinition = {
    "kind": "Option";
    "data": SettingsOption;
} | { "kind": "List"; "data": SettingsOptionList };


export interface SettingsOption {
    name: string;
    label: string;
    description: string;
    required: boolean;
    defaultValue: any;
    kind: SettingsOptionType;
}

export interface SettingsOptionList {
    name: string;
    label: string;
    description: string;
    required: boolean;
    defaultValue: any;
    template: Array<SettingsOption>;
}

export type SettingsOptionType =
    | { "kind": "string"; max_length: number | null; min_length: number | null }
    | { "kind": "float"; min: number | null; max: number | null }
    | { "kind": "integer"; min: bigint | null; max: bigint | null }
    | { "kind": "integer64"; min: string | null; max: string | null }
    | { "kind": "channel"; types: Array<ChannelType> | null }
    | {
        "kind": "channels";
        types: Array<ChannelType> | null;
        max_length: number | null;
        min_length: number | null;
    }
    | { "kind": "role"; assignable: boolean | null }
    | {
        "kind": "roles";
        assignable: boolean | null;
        max_length: number | null;
        min_length: number | null;
    };

export type ChannelType =
    | "Text"
    | "Voice"
    | "Category"
    | "News"
    | "StageVoice"
    | "NewsThread"
    | "PublicThread"
    | "PrivateThread"
    | "GuildDirectory"
    | "Forum"
    | { "Unknown": number };

export interface SettingsOptionValue {
    name: string;
    value: any;
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
    settings_values?: SettingsOptionValue[],
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
    // Only included on certain endpoints
    author?: BlUser,
    name: string,
    short_description: string,
    long_description: string,
    is_public: boolean,
    is_official: boolean,
    data: Variant,
    current_version: number,
    images: PluginImage[],
}

export interface PluginImage {
    plugin_id: number,
    image_id: string,
    created_at: string,
    description: string,
    position: number,
    kind: PluginImageKind,
    width: string,
    height: string,
}

export type PluginImageKind = "Icon" | "Showcase" | "Banner"

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

export interface BlUser {
    id: string;
    username: string;
    discriminator: string;
    avatar: string | null;

    is_bl_staff: boolean;
    is_bl_trusted: boolean;
}

export interface FullGuild {
    guild: Guild,
    channels: DiscordChannel[],
    roles: DiscordRole[],
}

export interface Guild {
    icon: string | null
    id: string
}

export interface DiscordChannel {
    id: string,
    // flags: Option<ChannelFlags>,
    guild_id: string,
    icon?: string,
    invitable?: boolean,
    kind: DiscordNumberedChannelTypes,
    // last_message_id: Option<Id<GenericMarker>>,
    // last_pin_timestamp: Option<Timestamp>,
    // managed: Option<bool>,
    // member: Option<ThreadMember>,
    // member_count: Option<i8>,
    // message_count: Option<u32>,
    name?: string,
    nsfw?: boolean,
    parent_id?: string,
    // permission_overwrites: Option<Vec<PermissionOverwrite>>,
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
    name: String,
    permissions: string,
    position: number,
    // flags: RoleFlags,
    // tags: Option<RoleTags>,
    // unicode_emoji: Option<String>,
}

