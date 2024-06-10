import { SupportedEncodeImageFormat } from "./generated/image/SupportedEncodeImageFormat";
import { SupportedImageFormat } from "./generated/image/SupportedImageFormat";
import { HttpClient } from "./httpclient"


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

    async transcode(intoFormat: SupportedEncodeImageFormat) {

    }

    dataUri(): string {
        return ""
    }
}

interface ImageProperties {
    format: SupportedImageFormat
    width: number
    height: number
}