import fetch from 'node-fetch';
import { ApiFetcher } from "botloader-common";

export function createFetcher(): ApiFetcher {
    return {
        fetch: async (path, opts) => await fetch(path, opts)
    };
}