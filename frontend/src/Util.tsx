import { ApiFetcher } from "botloader-common";

export function CreateFetcher(): ApiFetcher {
    return {
        fetch: async (path, opts) => await window.fetch(path, opts)
    }
}