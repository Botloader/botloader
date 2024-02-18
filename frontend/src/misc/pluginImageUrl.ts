import { BuildConfig } from "../BuildConfig";

export function pluginImageUrl(pluginId: number, imageId: string) {
    return `${BuildConfig.botloaderApiBase}/media/plugins/${pluginId}/images/${imageId}.webp`
}