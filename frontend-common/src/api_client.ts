/* eslint-disable @typescript-eslint/no-explicit-any */

import { operations, operationsMap } from "./generated/api";
import { FullGuild } from "./discord_models";

export interface EmptyResponse { }

export type Body = {
    body: any,
    kind: "json" | "custom"
}

// Typed wrappers over the generated operations: one method per operationId,
// e.g. client.operations.get_all_guild_scripts({ guild: "..." }).
type ClientOperations = {
    [K in keyof operations]: OperationFunc<K>;
}

type OperationFunc<K extends keyof operations> =
    OperationArgs<K> extends { __NO_BODY: true, __NO_PARAMS: true }
    ? () => Promise<ApiResult<OperationResponse<K>>>
    : (args: Omit<OperationArgs<K>, '__NO_BODY' | '__NO_PARAMS'>) => Promise<ApiResult<OperationResponse<K>>>;

type OperationArgs<K extends keyof operations> = OperationBody<K> & OperationParams<K>;

type OperationBody<K extends keyof operations> = operations[K]['requestBody'] extends { content: { "application/json": any } }
    ? {
        data: operations[K]['requestBody']['content']["application/json"]
    }
    : {
        __NO_BODY: true
    };

type OperationParams<K extends keyof operations> = Exclude<operations[K]['parameters']['query' | 'path'], undefined> extends never
    ? {
        __NO_PARAMS: true
    }
    : Exclude<operations[K]['parameters']['query' | 'path'], undefined>;

type InnerResponseContent<T> =
    T extends { content: { [contentType: string]: infer R } }
    ? R
    : EmptyResponse;

// The success (200) content of an operation; operations that only produce
// 204 get EmptyResponse. Error statuses become ApiError at runtime instead.
type OperationResponse<K extends keyof operations> =
    operations[K]['responses'] extends { 200: infer R }
    ? InnerResponseContent<R>
    : EmptyResponse;

export class ApiClient {
    token?: string;
    base: string;
    fetcher: ApiFetcher;
    operations: ClientOperations;

    // plug in either node-fetch or window.fetch depending on use context
    constructor(fetcher: ApiFetcher, base: string, token?: string) {
        this.token = token;
        this.base = base;
        this.fetcher = fetcher;
        this.operations = new Proxy({} as ClientOperations, {
            get: this.getOpMethod.bind(this),
        });
    }

    private getOpMethod(_target: any, prop: string | symbol): any {
        if (typeof prop === "symbol") {
            throw new Error("Symbol properties are not supported");
        }

        const op = operationsMap.find(op => op.name === prop);
        if (!op) {
            throw new Error(`Operation ${String(prop)} not found`);
        }

        return (args?: any) => this.opRequest(op.method, op.path, args);
    }

    private async opRequest(method: string, pathTemplate: string, args?: Record<string, any>): Promise<any> {
        const takenParams = new Set<string>();
        takenParams.add("data"); // 'data' is reserved for the request body

        const finalPath = pathTemplate.replace(/{(\w+)}/g, (_, paramName) => {
            const value = args?.[paramName];
            if (value === undefined) {
                throw new Error(`Missing value for path parameter: ${paramName}`);
            }
            takenParams.add(paramName);
            return encodeURIComponent(value);
        });

        // the rest of the args are query parameters
        const queryParams = Object.keys(args || {})
            .filter(key => !takenParams.has(key))
            .map(key => `${encodeURIComponent(key)}=${encodeURIComponent(args?.[key])}`)
            .join('&');

        const url = queryParams ? `${finalPath}?${queryParams}` : finalPath;

        const body: Body | undefined = args && "data" in args
            ? { kind: "json", body: args.data }
            : undefined;

        return await this.do(method.toUpperCase(), url, body);
    }

    async do<T>(method: string, path: string, body?: Body): Promise<ApiResult<T>> {
        let base = this.base;

        let headers = {};
        if (this.token) {
            headers = {
                Authorization: this.token,
                ...headers,
            };
        }

        let sendingBody = body?.body
        if (body && body.kind === "json") {
            headers = {
                "Content-Type": "application/json",
                ...headers,
            };
            sendingBody = JSON.stringify(body.body)
        }

        let response = await this.fetcher.fetch(base + path, {
            headers: headers,
            method: method,
            body: sendingBody,
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

    // hand-written: the generated FullGuild exposes channels/roles as untyped
    // json, this narrows it to the fields we actually use (see discord_models.ts)
    async getFullDiscordGuild(guildId: string): Promise<ApiResult<FullGuild>> {
        return await this.operations.get_full_guild({ guild: guildId }) as ApiResult<FullGuild>;
    }

    // hand-written: multipart upload, data should be a FormData but this
    // wrapper doesn't have DOM libs
    async addPluginImage(pluginId: number, data: any) {
        return await this.do(`POST`, `/api/user/plugins/${pluginId}/images`, {
            kind: "custom",
            body: data
        })
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

export enum ErrorCode {
    SessionExpired = 1,
    BadCsrfToken = 2,
    InternalError = 3,
    ValidationFailed = 4,
    NoActiveGuild = 5,
    NotGuildAdmin = 6,
    NoAccessToPlugin = 7,
    UserPluginLimitReached = 8,
    PluginNotFound = 9,
    GuildAlreadyHasPlugin = 10,
}
