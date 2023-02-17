import { GuildMetaConfig } from ".";
import { CreateScript, CurrentGuildsResponse, EmptyResponse, LoginResponse, Script, ScriptPlugin, SessionMeta, UpdateScript, User } from "./api_models";

/* eslint-disable @typescript-eslint/naming-convention */
export class ApiClient {
    token?: string;
    base: string;
    fetcher: ApiFetcher;

    // plug in either node-fetch or window.fetch depending on use context
    constructor(fetcher: ApiFetcher, base: string, token?: string) {
        this.token = token;
        this.base = base;
        this.fetcher = fetcher;
    }

    async do<T>(method: string, path: string, body?: any): Promise<ApiResult<T>> {
        let base = this.base;

        let headers = {};
        if (this.token) {
            headers = {
                Authorization: this.token,
                ...headers,
            };
        }

        if (body) {
            headers = {
                "Content-Type": "application/json",
                ...headers,
            };
        }

        let response = await this.fetcher.fetch(base + path, {
            headers: headers,
            method: method,
            body: body ? JSON.stringify(body) : undefined,
        });
        console.log(`Response status for ${path}: ${response.status}`);
        if (response.status === 204) {
            return {} as ApiResult<T>;
        }

        if (response.status !== 200) {
            let decoded: ApiErrorResponse = await response.json() as ApiErrorResponse;
            return {
                resp_code: response.status,
                is_error: true,
                response: decoded,
            };
        }

        return await response.json() as ApiResult<T>;
    }

    async get<T>(path: string,): Promise<ApiResult<T>> {
        return await this.do("GET", path);
    }

    async post<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("POST", path, body);
    }

    async delete<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("DELETE", path, body);
    }

    async put<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("PUT", path, body);
    }

    async patch<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("PATCH", path, body);
    }

    async getCurrentUser(): Promise<ApiResult<User>> {
        return await this.get("/api/current_user");
    }

    async getCurrentUserGuilds(): Promise<ApiResult<CurrentGuildsResponse>> {
        return await this.get("/api/guilds");
    }

    async getAllSessions(): Promise<ApiResult<SessionMeta[]>> {
        return await this.get("/api/sessions");
    }

    async logout(): Promise<ApiResult<{}>> {
        return await this.post("/api/logout");
    }

    async deleteSession(token: string): Promise<ApiResult<{}>> {
        return await this.delete("/api/sessions", {
            token: token,
        });
    }

    async deleteAllSessions(): Promise<ApiResult<{}>> {
        return await this.delete("/api/sessions/all");
    }

    async createApiToken(): Promise<ApiResult<SessionMeta>> {
        return await this.put("/api/sessions");
    }

    async confirmLogin(code: string, state: string): Promise<ApiResult<LoginResponse>> {
        return await this.post("/api/confirm_login", {
            code: code,
            state: state,
        });
    }

    async getUserPremiumSlots(): Promise<ApiResult<PremiumSlot[]>> {
        return await this.get("/api/premium_slots");
    }
    async updatePremiumSlotGuild(slotId: string, guildId: string | null): Promise<ApiResult<PremiumSlot>> {
        return await this.post(`/api/premium_slots/${slotId}/update_guild`, { guild_id: guildId });
    }

    async getAllScripts(guildId: string,): Promise<ApiResult<Script[]>> {
        return await this.get(`/api/guilds/${guildId}/scripts`);
    }

    async createScript(guildId: string, data: CreateScript): Promise<ApiResult<Script>> {
        return await this.put(`/api/guilds/${guildId}/scripts`, data);
    }

    async updateScript(guildId: string, id: number, data: UpdateScript): Promise<ApiResult<Script>> {
        return await this.patch(`/api/guilds/${guildId}/scripts/${id}`, data);
    }

    async delScript(guildId: string, id: number): Promise<ApiResult<EmptyResponse>> {
        return await this.delete(`/api/guilds/${guildId}/scripts/${id}`);
    }

    async reloadGuildVm(guildId: string): Promise<ApiResult<EmptyResponse>> {
        return await this.post(`/api/guilds/${guildId}/reload_vm`);
    }

    async getGuildMetaConfig(guildId: string): Promise<ApiResult<GuildMetaConfig>> {
        return await this.get(`/api/guilds/${guildId}/settings`);
    }

    async getNews(): Promise<ApiResult<NewsItem[]>> {
        return await this.get(`/api/news`);
    }

    async getGuildPremiumSlots(guildId: string): Promise<ApiResult<GuildPremiumSlot[]>> {
        return await this.get(`/api/guilds/${guildId}/premium_slots`);
    }

    async getPublishedPublicPlugin(): Promise<ApiResult<ScriptPlugin[]>> {
        return await this.get(`/api/plugins`);
    }

    async getCurrentUserPlugins(): Promise<ApiResult<ScriptPlugin[]>> {
        return await this.get(`/api/user/plugins`);
    }

    async getPlugin(scriptId: number): Promise<ApiResult<ScriptPlugin>> {
        return await this.get(`/api/plugins/${scriptId}`);
    }

    async createPlugin(params: {
        name: string,
        shortDescription?: string,
        longDescription?: string,
    }): Promise<ApiResult<ScriptPlugin>> {
        return await this.put(`/api/user/plugins`, {
            name: params.name,
            short_description: params.shortDescription,
            long_description: params.longDescription,
        });
    }

    async udpatePluginMeta(pluginId: number, params: {
        name?: string,
        shortDescription?: string,
        longDescription?: string,
        isPublic?: boolean,
    }): Promise<ApiResult<ScriptPlugin>> {
        return await this.patch(`/api/plugins/${pluginId}`, {
            name: params.name,
            short_description: params.shortDescription,
            long_description: params.longDescription,
            is_public: params.isPublic,
        });
    }

    async updateScriptPluginDevVersion(pluginId: number, params: {
        source: string,
    }): Promise<ApiResult<ScriptPlugin>> {
        return await this.put(`/api/plugins/${pluginId}/commit_script_dev_version`, {
            new_source: params.source
        });
    }

    async publishScriptPluginVersion(pluginId: number, params: {
        source: string,
    }): Promise<ApiResult<{}>> {
        return await this.put(`/api/plugins/${pluginId}/publish_script_version`, {
            new_source: params.source
        });
    }

    async addPluginToGuild(pluginId: number, guildId: string, params: {
        autoUpdate: boolean,
    }): Promise<ApiResult<ScriptPlugin>> {
        return await this.put(`/api/guilds/${guildId}/add_plugin`, {
            plugin_id: pluginId,
            auto_update: params.autoUpdate,
        });
    }

}

export type ApiResult<T> = T | ApiError;

export function isErrorResponse<T>(resp: ApiResult<T>): resp is ApiError {
    if (resp) {
        return (resp as ApiError).is_error !== undefined;
    } else {
        return false;
    }
}

export interface ApiError {
    resp_code: number,
    is_error: true,
    response?: ApiErrorResponse,
}

export interface ApiErrorResponse {
    code: number,
    description: string,
}

// just some simple abstractions so that we can use this in both a node and browser context
export interface ApiFetcher {
    fetch(path: string, opts: FetcherOpts): Promise<FetchResponse>;
}

export interface FetcherHeaders {
    [index: string]: string,
}

export interface FetcherOpts {
    headers: FetcherHeaders,
    method: string,
    body?: string,
}

export interface FetchResponse {
    json(): Promise<unknown>,
    status: number,
}

export interface PremiumSlot {
    id: number,
    title: string,
    user_id: string | null,
    message: string,
    source: string,
    source_id: string,
    tier: PremiumSlotTier,
    state: PremiumSlotState,
    created_at: string,
    updated_at: string,
    expires_at: string,
    manage_url: string,
    attached_guild_id: string | null,
}

export type PremiumSlotState =
    "Active" |
    "Cancelling" |
    "Cancelled" |
    "PaymentFailed";

export type PremiumSlotTier = "Lite" | "Premium";


export interface NewsItem {
    author: NewsAuthor,
    message_id: string,
    channel_id: string,
    channel_name: string,
    content: string,
    posted_at: number,
}

export interface NewsAuthor {
    username: string,
    avatar_url: string | null,
}

export interface GuildPremiumSlot {
    id: number,
    title: String,
    user_id: string | null,
    tier: PremiumSlotTier,
    created_at: string,
    updated_at: string,
    expires_at: string,
    attached_guild_id: string | null,
}