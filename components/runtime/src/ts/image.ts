import { HttpClient } from "./httpclient"

export type ImageEncodeSupportedFormats = "gif" | "png" | "jpeg" | "webp" | "rgba8"

export class Image {

    properties: ImageProperties;

    private imageData: Uint8Array;

    static fromBlob(data: Uint8Array) {

    }

    static async downloadFromUrl(url: string) {
        const request = await HttpClient.get(url)
        const data = await request.readAll()
        return this.fromBlob(data)
    }

    async resize(newWidth: number, newHeight: number) {

    }

    async transcode(intoFormat: ImageEncodeSupportedFormats) {

    }

    async dataUri(): string {
    }
}

op_image_transcode
op_imade_resize


interface ImageProperties {
    format: string
    width: number
    height: number
}