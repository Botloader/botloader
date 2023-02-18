import { GuildMetaConfig } from ".";
import { CreateScript, CurrentGuildsResponse, EmptyResponse, LoginResponse, Plugin, Script, ScriptPlugin, SessionMeta, UpdateScript, User } from "./api_models";

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
            return new ApiError(response.status, decoded);
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

    async getPublishedPublicPlugins(): Promise<ApiResult<Plugin[]>> {
        return await this.get(`/api/plugins`);
    }

    async getCurrentUserPlugins(): Promise<ApiResult<Plugin[]>> {
        return await this.get(`/api/user/plugins`);
    }

    async getPlugin(scriptId: number): Promise<ApiResult<Plugin>> {
        return await this.get(`/api/plugins/${scriptId}`);
    }

    async createPlugin(params: {
        name: string,
        short_description?: string,
        long_description?: string,
    }): Promise<ApiResult<Plugin>> {
        return await this.put(`/api/user/plugins`, params);
    }

    async updatePluginMeta(pluginId: number, params: {
        name?: string,
        short_description?: string,
        long_description?: string,
        is_public?: boolean,
    }): Promise<ApiResult<Plugin>> {
        return await this.patch(`/api/user/plugins/${pluginId}`, params);
    }

    async updateScriptPluginDevVersion(pluginId: number, params: {
        source: string,
    }): Promise<ApiResult<ScriptPlugin>> {
        return await this.patch(`/api/user/plugins/${pluginId}/dev_version`, {
            new_source: params.source
        });
    }

    async publishScriptPluginVersion(pluginId: number, params: {
        source: string,
    }): Promise<ApiResult<{}>> {
        return await this.post(`/api/user/plugins/${pluginId}/publish_script_version`, {
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

export function isErrorResponse(resp: any): resp is ApiError {
    return resp instanceof ApiError;
}

export class ApiError {
    resp_code: number;
    is_error: true = true;
    response?: ApiErrorResponse;

    constructor(resp_code: number, response?: ApiErrorResponse) {
        this.resp_code = resp_code;
        this.response = response;
    }

    getFieldError(field: string) {
        if (this.response?.code === 4 && this.response?.extra_data) {
            return this.response.extra_data.find((v) => v.field === field)?.msg
        }

        return undefined;
    }
}

export interface ApiErrorResponse {
    code: number,
    description: string,
    extra_data: null | ValidationError[],
}

export interface ValidationError {
    field: string,
    msg: string,
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