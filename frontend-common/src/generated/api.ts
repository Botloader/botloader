// This file is auto-generated. Do not edit manually. (see scripts/generate-webapi.sh)
export interface paths {
    "/api/plugins": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_published_public_plugins"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/plugins/{plugin_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_plugin"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/news": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_news"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/confirm_login": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["confirm_login"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/reload_vm": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["reload_guild_vm"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/settings": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_guild_settings"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/premium_slots": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_guild_premium_slots"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/scripts": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_all_guild_scripts"];
        put: operations["create_guild_script"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/scripts_with_plugins": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_all_guild_scripts_with_plugins"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/scripts/{script_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["delete_guild_script"];
        options?: never;
        head?: never;
        patch: operations["update_guild_script"];
        trace?: never;
    };
    "/api/guilds/{guild}/scripts/{script_id}/validate_settings": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["validate_script_settings"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/scripts/{script_id}/update_plugin": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["update_script_plugin"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/add_plugin": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["guild_add_plugin"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds/{guild}/full_guild": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_full_guild"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/admin/vm_workers": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_worker_statuses"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/admin/guild/{guild_id}/status": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_guild_status"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/guilds": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_user_guilds"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/premium_slots/{slot_id}/update_guild": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["update_premium_slot_guild"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/premium_slots": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["list_user_premium_slots"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/sessions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_all_sessions"];
        put: operations["create_api_token"];
        post?: never;
        delete: operations["del_session"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/sessions/all": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["del_all_sessions"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/current_user": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_current_user"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/user/plugins": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_user_plugins"];
        put: operations["create_plugin"];
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/user/plugins/{plugin_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch: operations["update_plugin_meta"];
        trace?: never;
    };
    "/api/user/plugins/{plugin_id}/dev_version": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch: operations["update_plugin_dev_source"];
        trace?: never;
    };
    "/api/user/plugins/{plugin_id}/publish_script_version": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["publish_plugin_version"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/user/plugins/{plugin_id}/images": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["add_plugin_image"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/user/plugins/{plugin_id}/images/{image_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        delete: operations["delete_plugin_image"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/logout": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["logout"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/stripe/customer_portal": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["create_customer_portal_session"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/api/stripe/create_checkout_session": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post: operations["create_checkout_session"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        ApiVmWorkerStatus: {
            /** Format: uint32 */
            worker_id: number;
            currently_claimed_by_guild_id?: string | null;
            last_claimed_by_guild_id?: string | null;
            /** Format: uint64 */
            claimed_last_ms_ago: number;
            /** Format: uint64 */
            returned_last_ms_ago: number;
        };
        /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
        ApiErrorBody: {
            description: string;
            /** Format: uint32 */
            code: number;
            extra_data?: unknown;
        };
        GuildIdParam: {
            /** Format: uint64 */
            guild_id: number;
        };
        ApiGuildStatusResponse: {
            guild_id: string;
            /** Format: uint32 */
            current_claimed_worker_id?: number | null;
            /** Format: uint32 */
            last_claimed_worker_id?: number | null;
            /** Format: uint64 */
            claimed_last_since_ms: number;
            /** Format: uint64 */
            returned_last_since_ms: number;
            /** Format: uint32 */
            pending_acks: number;
        };
        /** @description A guilds config, for storing core botloader settings */
        GuildMetaConfig: {
            guild_id: string;
            error_channel_id?: string | null;
        };
        GuildPremiumSlot: {
            title: string;
            /** Format: uint64 */
            id: number;
            user_id?: string | null;
            tier: components["schemas"]["PremiumSlotTier"];
            /** Format: date-time */
            created_at: string;
            /** Format: date-time */
            updated_at: string;
            /** Format: date-time */
            expires_at: string;
            attached_guild_id?: string | null;
        };
        /** @enum {string} */
        PremiumSlotTier: "Lite" | "Premium";
        /** @description Struct you get back from the store */
        Script: {
            /** Format: uint64 */
            id: number;
            name: string;
            original_source: string;
            enabled: boolean;
            contributes: components["schemas"]["ScriptContributes"];
            /** Format: uint64 */
            plugin_id?: number | null;
            plugin_auto_update?: boolean | null;
            /** Format: uint32 */
            plugin_version_number?: number | null;
            settings_definitions?: components["schemas"]["SettingsOptionDefinition"][] | null;
            settings_values: components["schemas"]["SettingsOptionValue"][];
        };
        /** @description Contribution points for a scripts, e.g triggers, commands etc */
        ScriptContributes: {
            commands: unknown[];
            interval_timers: components["schemas"]["IntervalTimerContrib"][];
        };
        IntervalTimerContrib: {
            name: string;
            interval: components["schemas"]["IntervalType"];
            /** Format: uint64 */
            plugin_id?: number | null;
        };
        IntervalType: {
            /** Format: uint64 */
            Minutes: number;
        } | {
            Cron: string;
        };
        SettingsOptionDefinition: {
            /** @constant */
            kind: "Option";
            data: components["schemas"]["SettingsOption"];
        } | {
            /** @constant */
            kind: "List";
            data: components["schemas"]["SettingsOptionList"];
        };
        SettingsOption: {
            description: string;
            name: string;
            label: string;
            required: boolean;
            defaultValue?: unknown;
            kind: components["schemas"]["SettingsOptionType"];
        };
        SettingsOptionType: {
            /** Format: uint32 */
            max_length?: number | null;
            /** Format: uint32 */
            min_length?: number | null;
            /** @constant */
            kind: "string";
        } | {
            /** Format: double */
            min?: number | null;
            /** Format: double */
            max?: number | null;
            /** @constant */
            kind: "float";
        } | {
            min?: components["schemas"]["NotBigI64"] | null;
            max?: components["schemas"]["NotBigI64"] | null;
            /** @constant */
            kind: "integer";
        } | {
            min?: string | null;
            max?: string | null;
            /** @constant */
            kind: "integer64";
        } | {
            /** @constant */
            kind: "boolean";
        } | {
            options: components["schemas"]["SettingsStringSelectOption"][];
            /** @constant */
            kind: "customStringSelect";
        } | {
            options: components["schemas"]["SettingsStringSelectOption"][];
            /** Format: uint32 */
            max_selected?: number | null;
            /** Format: uint32 */
            min_selected?: number | null;
            /** @constant */
            kind: "customStringMultiSelect";
        } | {
            options: components["schemas"]["SettingsNumberSelectOption"][];
            /** @constant */
            kind: "customNumberSelect";
        } | {
            options: components["schemas"]["SettingsNumberSelectOption"][];
            /** Format: uint32 */
            max_selected?: number | null;
            /** Format: uint32 */
            min_selected?: number | null;
            /** @constant */
            kind: "customNumberMultiSelect";
        } | {
            types?: components["schemas"]["ChannelType"][] | null;
            /** @constant */
            kind: "channel";
        } | {
            types?: components["schemas"]["ChannelType"][] | null;
            /** Format: uint32 */
            max_length?: number | null;
            /** Format: uint32 */
            min_length?: number | null;
            /** @constant */
            kind: "channels";
        } | {
            assignable?: boolean | null;
            /** @constant */
            kind: "role";
        } | {
            assignable?: boolean | null;
            /** Format: uint32 */
            max_length?: number | null;
            /** Format: uint32 */
            min_length?: number | null;
            /** @constant */
            kind: "roles";
        };
        /** Format: int64 */
        NotBigI64: number;
        SettingsStringSelectOption: {
            label: string;
            value: string;
        };
        SettingsNumberSelectOption: {
            label: string;
            /** Format: double */
            value: number;
        };
        /** @enum {string} */
        ChannelType: "Text" | "Voice" | "Category" | "News" | "Store" | "StageVoice" | "NewsThread" | "PublicThread" | "PrivateThread" | "GuildDirectory" | "Forum" | "Media";
        SettingsOptionList: {
            description: string;
            name: string;
            label: string;
            required: boolean;
            defaultValue?: unknown;
            template: components["schemas"]["SettingsOption"][];
        };
        SettingsOptionValue: {
            name: string;
            value: unknown;
        };
        CreateRequestData: {
            name: string;
            original_source: string;
            enabled: boolean;
        };
        GetScriptsWithPluginsResponse: {
            scripts: components["schemas"]["Script"][];
            plugins: components["schemas"]["Plugin"][];
        };
        Plugin: {
            /** Format: uint64 */
            id: number;
            /** Format: date-time */
            created_at: string;
            author_id: string;
            name: string;
            short_description: string;
            long_description: string;
            is_public: boolean;
            is_official: boolean;
            is_published: boolean;
            /** Format: uint32 */
            current_version: number;
            /** Format: date-time */
            published_version_updated_at?: string | null;
            /** Format: uint32 */
            installed_guilds?: number | null;
            /** Format: date-time */
            installed_guilds_updated_at?: string | null;
            discord_thread_id?: string | null;
            images: components["schemas"]["PluginImage"][];
            data: components["schemas"]["PluginData"];
        };
        PluginImage: {
            description: string;
            /** Format: uint64 */
            plugin_id: number;
            /** Format: uuid */
            image_id: string;
            /** Format: date-time */
            created_at: string;
            /** Format: int32 */
            position: number;
            kind: components["schemas"]["PluginImageKind"];
            /** Format: uint32 */
            width: number;
            /** Format: uint32 */
            height: number;
        };
        /** @enum {string} */
        PluginImageKind: "Icon" | "Banner" | "Showcase";
        PluginData: {
            /** @constant */
            plugin_type: "ScriptPlugin";
        } & components["schemas"]["ScriptPluginData"];
        ScriptPluginData: {
            published_version?: string | null;
            /** Format: date-time */
            published_version_updated_at?: string | null;
            dev_version?: string | null;
            /** Format: date-time */
            dev_version_updated_at?: string | null;
        };
        GuildScriptPathParams: {
            /** Format: uint64 */
            script_id: number;
        };
        UpdateRequestData: {
            /** @default null */
            name?: string | null;
            /** @default null */
            original_source?: string | null;
            /** @default null */
            enabled?: boolean | null;
            /** @default null */
            settings_values?: components["schemas"]["SettingsOptionValue"][] | null;
        };
        GuildAddPluginData: {
            /** Format: uint64 */
            plugin_id: number;
            auto_update: boolean;
        };
        FullGuild: {
            guild: unknown;
            channels: unknown[];
            roles: unknown[];
        };
        GuildList: {
            guilds: components["schemas"]["GuildListEntry"][];
        };
        GuildListEntry: {
            connected: boolean;
            guild: components["schemas"]["CurrentUserGuild"];
        };
        /** @description Mirror of [twilight_model::user::CurrentUserGuild] */
        CurrentUserGuild: {
            id: string;
            name: string;
            icon?: string | null;
            owner: boolean;
            /** @description Permissions bitflags serialized as a string */
            permissions: string;
            features: string[];
        };
        UpdateSlotPathParams: {
            /** Format: uint64 */
            slot_id: number;
        };
        UpdateSlotGuildBody: {
            guild_id?: string | null;
        };
        PremiumSlot: {
            title: string;
            /** Format: uint64 */
            id: number;
            user_id?: string | null;
            message: string;
            source: string;
            source_id: string;
            tier: components["schemas"]["PremiumSlotTier"];
            state: components["schemas"]["PremiumSlotState"];
            /** Format: date-time */
            created_at: string;
            /** Format: date-time */
            updated_at: string;
            /** Format: date-time */
            expires_at: string;
            manage_url: string;
            attached_guild_id?: string | null;
        };
        /** @enum {string} */
        PremiumSlotState: "Active" | "Cancelling" | "Cancelled" | "PaymentFailed";
        SessionMeta: {
            kind: components["schemas"]["SessionType"];
            /** Format: date-time */
            created_at: string;
        };
        /** @enum {string} */
        SessionType: "User" | "ApiKey";
        DelSessionPayload: {
            token: string;
        };
        SessionMetaWithKey: {
            kind: components["schemas"]["SessionType"];
            /** Format: date-time */
            created_at: string;
            token: string;
        };
        /** @description Mirror of [twilight_model::user::CurrentUser] */
        CurrentUser: {
            id: string;
            username: string;
            discriminator: string;
            avatar?: string | null;
            bot: boolean;
            mfa_enabled: boolean;
            locale?: string | null;
            verified?: boolean | null;
            email?: string | null;
            /** Format: uint64 */
            flags?: number | null;
            /** Format: uint8 */
            premium_type?: number | null;
            /** Format: uint64 */
            public_flags?: number | null;
            /** Format: uint32 */
            accent_color?: number | null;
            banner?: string | null;
        };
        PluginResponse: {
            /** Format: uint64 */
            id: number;
            /** Format: date-time */
            created_at: string;
            author_id: string;
            name: string;
            short_description: string;
            long_description: string;
            is_public: boolean;
            is_official: boolean;
            is_published: boolean;
            /** Format: uint32 */
            current_version: number;
            /** Format: date-time */
            published_version_updated_at?: string | null;
            /** Format: uint32 */
            installed_guilds?: number | null;
            /** Format: date-time */
            installed_guilds_updated_at?: string | null;
            discord_thread_id?: string | null;
            images: components["schemas"]["PluginImage"][];
            data: components["schemas"]["PluginData"];
            author: components["schemas"]["User"];
        };
        User: {
            id: string;
            username: string;
            discriminator: string;
            avatar?: string | null;
            is_bl_staff: boolean;
            is_bl_trusted: boolean;
        };
        CreatePluginBody: {
            name: string;
            short_description: string;
            long_description: string;
        };
        UpdatePluginMetaRequest: {
            name?: string | null;
            short_description?: string | null;
            long_description?: string | null;
            is_public?: boolean | null;
            is_published?: boolean | null;
        };
        UpdatePluginDevSourceRequest: {
            new_source: string;
        };
        PublishPluginVersionData: {
            new_source: string;
        };
        ImageParam: {
            /** Format: uuid */
            image_id: string;
        };
        UrlResponse: {
            url: string;
        };
        CreateCheckoutSessionBody: {
            tier: components["schemas"]["PremiumSlotTier"];
        };
        NewsItem: {
            author: components["schemas"]["NewsAuthor"];
            message_id: string;
            channel_id: string;
            channel_name: string;
            content: string;
            /** Format: int64 */
            posted_at: number;
        };
        NewsAuthor: {
            username: string;
            avatar_url?: string | null;
        };
        ConfirmLoginQuery: {
            code: string;
            state: string;
        };
        ConfirmLoginSuccess: {
            user: components["schemas"]["CurrentUser"];
            token: string;
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type ApiVmWorkerStatus = components['schemas']['ApiVmWorkerStatus'];
export type ApiErrorBody = components['schemas']['ApiErrorBody'];
export type GuildIdParam = components['schemas']['GuildIdParam'];
export type ApiGuildStatusResponse = components['schemas']['ApiGuildStatusResponse'];
export type GuildMetaConfig = components['schemas']['GuildMetaConfig'];
export type GuildPremiumSlot = components['schemas']['GuildPremiumSlot'];
export type PremiumSlotTier = components['schemas']['PremiumSlotTier'];
export type Script = components['schemas']['Script'];
export type ScriptContributes = components['schemas']['ScriptContributes'];
export type IntervalTimerContrib = components['schemas']['IntervalTimerContrib'];
export type IntervalType = components['schemas']['IntervalType'];
export type SettingsOptionDefinition = components['schemas']['SettingsOptionDefinition'];
export type SettingsOption = components['schemas']['SettingsOption'];
export type SettingsOptionType = components['schemas']['SettingsOptionType'];
export type NotBigI64 = components['schemas']['NotBigI64'];
export type SettingsStringSelectOption = components['schemas']['SettingsStringSelectOption'];
export type SettingsNumberSelectOption = components['schemas']['SettingsNumberSelectOption'];
export type ChannelType = components['schemas']['ChannelType'];
export type SettingsOptionList = components['schemas']['SettingsOptionList'];
export type SettingsOptionValue = components['schemas']['SettingsOptionValue'];
export type CreateRequestData = components['schemas']['CreateRequestData'];
export type GetScriptsWithPluginsResponse = components['schemas']['GetScriptsWithPluginsResponse'];
export type Plugin = components['schemas']['Plugin'];
export type PluginImage = components['schemas']['PluginImage'];
export type PluginImageKind = components['schemas']['PluginImageKind'];
export type PluginData = components['schemas']['PluginData'];
export type ScriptPluginData = components['schemas']['ScriptPluginData'];
export type GuildScriptPathParams = components['schemas']['GuildScriptPathParams'];
export type UpdateRequestData = components['schemas']['UpdateRequestData'];
export type GuildAddPluginData = components['schemas']['GuildAddPluginData'];
export type FullGuild = components['schemas']['FullGuild'];
export type GuildList = components['schemas']['GuildList'];
export type GuildListEntry = components['schemas']['GuildListEntry'];
export type CurrentUserGuild = components['schemas']['CurrentUserGuild'];
export type UpdateSlotPathParams = components['schemas']['UpdateSlotPathParams'];
export type UpdateSlotGuildBody = components['schemas']['UpdateSlotGuildBody'];
export type PremiumSlot = components['schemas']['PremiumSlot'];
export type PremiumSlotState = components['schemas']['PremiumSlotState'];
export type SessionMeta = components['schemas']['SessionMeta'];
export type SessionType = components['schemas']['SessionType'];
export type DelSessionPayload = components['schemas']['DelSessionPayload'];
export type SessionMetaWithKey = components['schemas']['SessionMetaWithKey'];
export type CurrentUser = components['schemas']['CurrentUser'];
export type PluginResponse = components['schemas']['PluginResponse'];
export type User = components['schemas']['User'];
export type CreatePluginBody = components['schemas']['CreatePluginBody'];
export type UpdatePluginMetaRequest = components['schemas']['UpdatePluginMetaRequest'];
export type UpdatePluginDevSourceRequest = components['schemas']['UpdatePluginDevSourceRequest'];
export type PublishPluginVersionData = components['schemas']['PublishPluginVersionData'];
export type ImageParam = components['schemas']['ImageParam'];
export type UrlResponse = components['schemas']['UrlResponse'];
export type CreateCheckoutSessionBody = components['schemas']['CreateCheckoutSessionBody'];
export type NewsItem = components['schemas']['NewsItem'];
export type NewsAuthor = components['schemas']['NewsAuthor'];
export type ConfirmLoginQuery = components['schemas']['ConfirmLoginQuery'];
export type ConfirmLoginSuccess = components['schemas']['ConfirmLoginSuccess'];
export type $defs = Record<string, never>;
export interface operations {
    get_published_public_plugins: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PluginResponse"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_plugin: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                plugin_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PluginResponse"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_news: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NewsItem"][];
                };
            };
        };
    };
    confirm_login: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ConfirmLoginQuery"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ConfirmLoginSuccess"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    reload_guild_vm: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_guild_settings: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description A guilds config, for storing core botloader settings */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GuildMetaConfig"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_guild_premium_slots: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GuildPremiumSlot"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_all_guild_scripts: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    create_guild_script: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateRequestData"];
            };
        };
        responses: {
            /** @description Struct you get back from the store */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_all_guild_scripts_with_plugins: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GetScriptsWithPluginsResponse"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    delete_guild_script: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                script_id: number;
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Struct you get back from the store */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    update_guild_script: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                script_id: number;
                guild: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateRequestData"];
            };
        };
        responses: {
            /** @description Struct you get back from the store */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    validate_script_settings: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                script_id: number;
                guild: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateRequestData"];
            };
        };
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    update_script_plugin: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                script_id: number;
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Struct you get back from the store */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    guild_add_plugin: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["GuildAddPluginData"];
            };
        };
        responses: {
            /** @description Struct you get back from the store */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Script"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_full_guild: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["FullGuild"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_worker_statuses: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiVmWorkerStatus"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_guild_status: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                guild_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiGuildStatusResponse"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    list_user_guilds: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GuildList"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    update_premium_slot_guild: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                slot_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdateSlotGuildBody"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PremiumSlot"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    list_user_premium_slots: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PremiumSlot"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_all_sessions: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SessionMeta"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    create_api_token: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SessionMetaWithKey"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    del_session: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["DelSessionPayload"];
            };
        };
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    del_all_sessions: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_current_user: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Mirror of [twilight_model::user::CurrentUser] */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CurrentUser"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    get_user_plugins: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PluginResponse"][];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    create_plugin: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreatePluginBody"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Plugin"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    update_plugin_meta: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                plugin_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdatePluginMetaRequest"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Plugin"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    update_plugin_dev_source: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                plugin_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["UpdatePluginDevSourceRequest"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Plugin"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    publish_plugin_version: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                plugin_id: number;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PublishPluginVersionData"];
            };
        };
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    add_plugin_image: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                plugin_id: number;
            };
            cookie?: never;
        };
        /** @description multipart form data */
        requestBody: {
            content: {
                "multipart/form-data": unknown[];
            };
        };
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    delete_plugin_image: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                image_id: string;
                plugin_id: number;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    logout: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": null;
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    create_customer_portal_session: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UrlResponse"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
    create_checkout_session: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateCheckoutSessionBody"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["UrlResponse"];
                };
            };
            /** @description Shape of the json body produced by [ApiErrorResponse::into_response], for schema generation */
            default: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ApiErrorBody"];
                };
            };
        };
    };
}


export const operationsMap = [
    {
        "method": "get",
        "name": "get_published_public_plugins",
        "path": "/api/plugins"
    },
    {
        "method": "get",
        "name": "get_plugin",
        "path": "/api/plugins/{plugin_id}"
    },
    {
        "method": "get",
        "name": "get_news",
        "path": "/api/news"
    },
    {
        "method": "post",
        "name": "confirm_login",
        "path": "/api/confirm_login"
    },
    {
        "method": "post",
        "name": "reload_guild_vm",
        "path": "/api/guilds/{guild}/reload_vm"
    },
    {
        "method": "get",
        "name": "get_guild_settings",
        "path": "/api/guilds/{guild}/settings"
    },
    {
        "method": "get",
        "name": "get_guild_premium_slots",
        "path": "/api/guilds/{guild}/premium_slots"
    },
    {
        "method": "get",
        "name": "get_all_guild_scripts",
        "path": "/api/guilds/{guild}/scripts"
    },
    {
        "method": "put",
        "name": "create_guild_script",
        "path": "/api/guilds/{guild}/scripts"
    },
    {
        "method": "get",
        "name": "get_all_guild_scripts_with_plugins",
        "path": "/api/guilds/{guild}/scripts_with_plugins"
    },
    {
        "method": "delete",
        "name": "delete_guild_script",
        "path": "/api/guilds/{guild}/scripts/{script_id}"
    },
    {
        "method": "patch",
        "name": "update_guild_script",
        "path": "/api/guilds/{guild}/scripts/{script_id}"
    },
    {
        "method": "post",
        "name": "validate_script_settings",
        "path": "/api/guilds/{guild}/scripts/{script_id}/validate_settings"
    },
    {
        "method": "post",
        "name": "update_script_plugin",
        "path": "/api/guilds/{guild}/scripts/{script_id}/update_plugin"
    },
    {
        "method": "post",
        "name": "guild_add_plugin",
        "path": "/api/guilds/{guild}/add_plugin"
    },
    {
        "method": "get",
        "name": "get_full_guild",
        "path": "/api/guilds/{guild}/full_guild"
    },
    {
        "method": "get",
        "name": "get_worker_statuses",
        "path": "/api/admin/vm_workers"
    },
    {
        "method": "get",
        "name": "get_guild_status",
        "path": "/api/admin/guild/{guild_id}/status"
    },
    {
        "method": "get",
        "name": "list_user_guilds",
        "path": "/api/guilds"
    },
    {
        "method": "post",
        "name": "update_premium_slot_guild",
        "path": "/api/premium_slots/{slot_id}/update_guild"
    },
    {
        "method": "get",
        "name": "list_user_premium_slots",
        "path": "/api/premium_slots"
    },
    {
        "method": "get",
        "name": "get_all_sessions",
        "path": "/api/sessions"
    },
    {
        "method": "put",
        "name": "create_api_token",
        "path": "/api/sessions"
    },
    {
        "method": "delete",
        "name": "del_session",
        "path": "/api/sessions"
    },
    {
        "method": "delete",
        "name": "del_all_sessions",
        "path": "/api/sessions/all"
    },
    {
        "method": "get",
        "name": "get_current_user",
        "path": "/api/current_user"
    },
    {
        "method": "get",
        "name": "get_user_plugins",
        "path": "/api/user/plugins"
    },
    {
        "method": "put",
        "name": "create_plugin",
        "path": "/api/user/plugins"
    },
    {
        "method": "patch",
        "name": "update_plugin_meta",
        "path": "/api/user/plugins/{plugin_id}"
    },
    {
        "method": "patch",
        "name": "update_plugin_dev_source",
        "path": "/api/user/plugins/{plugin_id}/dev_version"
    },
    {
        "method": "post",
        "name": "publish_plugin_version",
        "path": "/api/user/plugins/{plugin_id}/publish_script_version"
    },
    {
        "method": "post",
        "name": "add_plugin_image",
        "path": "/api/user/plugins/{plugin_id}/images"
    },
    {
        "method": "delete",
        "name": "delete_plugin_image",
        "path": "/api/user/plugins/{plugin_id}/images/{image_id}"
    },
    {
        "method": "post",
        "name": "logout",
        "path": "/api/logout"
    },
    {
        "method": "post",
        "name": "create_customer_portal_session",
        "path": "/api/stripe/customer_portal"
    },
    {
        "method": "post",
        "name": "create_checkout_session",
        "path": "/api/stripe/create_checkout_session"
    }
]