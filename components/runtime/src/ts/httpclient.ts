import { OpWrappers } from "./op_wrappers";
import { AsyncReadCloser, AsyncReader, NativeReader } from "./unstable/streams";
import { decodeText, encodeText } from "./core_util";

export namespace HttpClient {

    /**
     * Construct a new request
     * 
     * @param method Request method
     * @param path Request path
     * @param init Additional options
     * @returns A constructed request, this still needs to be sent with a optional body
     */
    export function request(method: string, path: string, init?: RequestInit) {
        let opts = {
            ...(init ?? {})
        }

        return new Request(method, path, opts);
    }

    /**
     * Construct a new GET request
     * 
     * @category Helpers
     */
    export function get(path: string, init?: RequestInit) {
        return request("GET", path, init);
    }

    /**
     * Construct a new POST request
     * 
     * @category Helpers
     */
    export function post(path: string, init?: RequestInit) {
        return request("POST", path, init);
    }

    /**
     * Construct a new HEAD request
     * 
     * @category Helpers
     */
    export function head(path: string, init?: RequestInit) {
        return request("HEAD", path, init);
    }

    /**
     * Construct a new PATCH request
     * 
     * @category Helpers
     */
    export function patch(path: string, init?: RequestInit) {
        return request("PATCH", path, init);
    }

    /**
     * Construct a new PUT request
     * 
     * @category Helpers
     */
    export function put(path: string, init?: RequestInit) {
        return request("PUT", path, init);
    }

    /**
     * Construct a new DELETE request
     * 
     * @category Helpers
     */
    export function del(path: string, init?: RequestInit) {
        return request("DELETE", path, init);
    }

    /**
     * Construct a new OPTIONS request
     * 
     * @category Helpers
     */
    export function options(path: string, init?: RequestInit) {
        return request("OPTIONS", path, init);
    }

    /**
     * Construct a new TRACE request
     * 
     * @category Helpers
     */
    export function trace(path: string, init?: RequestInit) {
        return request("TRACE", path, init);
    }

    /**
     * Construct a new CONNECT request
     * 
     * @category Helpers
     */
    export function connect(path: string, init?: RequestInit) {
        return request("CONNECT", path, init);
    }

    export interface RequestInit {
        /**
         * The script that issued this request, used for analytics.
         * 
         * @internal
         */
        scriptId?: number;

        /**
         * A Headers object, an object literal, or an array of two-item arrays to set
         * request's headers.
         */
        headers?: Record<string, string>;

        /**
         * A string indicating whether request follows redirects, results in an error
         * upon encountering a redirect, or returns the redirect (in an opaque
         * fashion). Sets request's redirect.
         */
        // redirect?: "follow" | "manual" | "error";
    }

    /**
     * A constructed http request that has yet to be sent.
     * 
     * Use one of the `send` methods to send it.
     */
    export class Request {
        /** {@inheritdoc RequestInit.scriptId} */
        scriptId?: number;

        /** {@inheritdoc RequestInit.headers} */
        headers?: Record<string, string>;

        // /** {@inheritdoc RequestInit.redirect} */
        // redirect?: "follow" | "manual" | "error";

        path: string;
        method: string;

        constructor(method: string, path: string, init?: RequestInit) {
            this.path = path;
            this.method = method;

            this.headers = init?.headers;
            // this.redirect = init?.redirect;
        }

        /**
         * Send this request with a optional body.
         * 
         * Note: if you dont consume the response body you should call `Response.body.close()`
         * (this is done automatically after 30 seconds if you don't, but if you're sending a lot of requests its better to do it yourself)
         * 
         * @param body The request body, see {@link sendJson} and {@link sendText} for helpers
         * @returns A promise that resolbes with the response
         */
        async send(body?: RequestBody): Promise<Response> {
            let reqBodyRid: number | undefined = undefined;
            if (body) {
                this.headers = this.headers ?? {}
                body.setHeaders(this.headers);

                reqBodyRid = OpWrappers.http.createRequestStream();
                this.writeBody(body, reqBodyRid).finally(() => Deno.core.tryClose(reqBodyRid!));
            }

            let resp = await OpWrappers.http.requestSend({
                headers: this.headers ?? {},
                method: this.method,
                path: this.path,
                scriptId: this.scriptId ?? 0,
                bodyResourceId: reqBodyRid,
            });

            let respBody = new NativeReader(resp.bodyResourceId);

            return new Response(resp.statusCode, resp.headers, respBody);
        }

        then(cb: (val: Response) => any) {
            return this.send().then(cb);
        }

        private async writeBody(body: RequestBody, rid: number) {
            let buf = new Uint8Array(1024 * 10);
            while (true) {
                let read = await body.read(buf);
                if (read > 0) {
                    let written = await Deno.core.write(rid, buf.subarray(0, read));
                    if (written < read) {
                        return
                    }
                }

                if (read === 0) {
                    return
                }
            }
        }

        async sendJson(data: any) {
            const serialized = encodeText(JSON.stringify(data));
            return this.send(new Uint8ArrayBody(serialized, "application/json"));
        }

        async sendText(data: string) {
            const encoded = encodeText(data);
            return this.send(new Uint8ArrayBody(encoded, "text/plain"));
        }
    }


    export interface RequestBody extends AsyncReader {
        setHeaders(headers: Record<string, string>): void;
    }

    export class Uint8ArrayBody implements RequestBody {
        contentType?: string;
        remainingData: Uint8Array;

        constructor(data: Uint8Array, contentType?: string) {
            this.contentType = contentType;
            this.remainingData = data;
        }


        setHeaders(headers: Record<string, string>) {
            if (this.contentType) {
                headers["Content-Type"] = this.contentType;
            }
        }

        async read(buf: Uint8Array) {
            if (this.remainingData.length > 0) {
                const readValues = this.remainingData.subarray(0, buf.length);
                buf.set(readValues, 0);
                this.remainingData = this.remainingData.subarray(readValues.length);
                return readValues.length;
            }

            return 0;
        }

    }

    /**
     * A response to a http request.
     * 
     * Note: if you dont consume the response body you should call `Response.body.close()`
     * (this is done automatically after 30 seconds if you don't, but if you're sending a lot of requests its better to do it yourself)
     */
    export class Response {
        body?: AsyncReadCloser;
        headers: Record<string, string>;
        statusCode: number;

        constructor(statusCode: number, headers: Record<string, string>, body?: AsyncReadCloser) {
            this.statusCode = statusCode;
            this.headers = headers;
            this.body = body;
        }

        /**
         * Read the entire response body and return the raw Uint8Array for it.
         */
        async readAll(): Promise<Uint8Array> {
            if (!this.body) {
                throw new Error("no response body")
            }

            let curBuffer = new Uint8Array();
            let readBuf = new Uint8Array(1024 * 10);

            while (true) {
                const n = await this.body.read(readBuf);
                const newBuffer = new Uint8Array(curBuffer.length + n);
                newBuffer.set(curBuffer, 0);
                newBuffer.set(readBuf.slice(0, n), curBuffer.length);
                curBuffer = newBuffer;

                if (n === 0) {
                    break;
                }
            }

            return curBuffer;
        }

        /**
         * Read and decode the response body as json
         */
        async json<T>(): Promise<T> {
            let fullBody = decodeText(await this.readAll());
            return JSON.parse(fullBody);
        }

        /**
         * Read and decode the reponse body as utf8 encoded text
         */
        async text(): Promise<string> {
            let buf = await this.readAll();
            return decodeText(buf);
        }
    }
}
