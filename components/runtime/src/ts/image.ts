import { base64Encode } from "./core_util";
import { SupportedEncodeImageFormat } from "./generated/image/SupportedEncodeImageFormat";
import { SupportedImageFormat } from "./generated/image/SupportedImageFormat";
import { HttpClient } from "./httpclient"
import { OpWrappers } from "./op_wrappers";

/**
 * The image class provides basic image manipulation and inspection.
 * 
 * This API is currently considered somewhat unstable, there are bugs here as working with a lot of image formats (and animations) is surprisingly hard
 * 
 * Decoding Formats supported are:
 * - png
 * - jpeg
 * - gif
 * - webp
 * - pnm
 * - tiff
 * - tga
 * - bmp
 * - ico
 * - hdr
 * - openexr
 * - qoi
 * - avif
 * 
 * Encoding formats supported:
 * - png (encoding animations not supported yet)
 * - jpeg
 * - gif
 * - webp (encoding animations not supported yet)
 *
 * There are known bugs and issues currently with some of these formats so consider this API to be in beta. 
 * 
 * @beta
 * 
 */
export class Image {

    readonly properties: ImageProperties;

    readonly data: ArrayBuffer;

    constructor(data: ArrayBuffer, properties?: ImageProperties) {
        this.data = data
        this.properties = properties ? properties : OpWrappers.opImageProperties(data);
    }

    static async downloadFromUrl(url: string) {
        const request = await HttpClient.get(url)
        const data = await request.readAll()
        return new Image(data.buffer)
    }

    async resize(newWidth: number, newHeight: number): Promise<Image> {
        if (!isEncodable(this.properties.formatName)) {
            throw new Error(`Botloader cannot encode ${this.properties.formatName} images yet`)
        }

        return this.transcode({
            outputFormat: this.properties.formatName,
            resizeWidth: newWidth,
            resizeHeight: newHeight,
        })
    }

    async transcode(options: ImageTranscodeOptions) {
        const resize = (options.resizeHeight || options.resizeWidth)
            ? [options.resizeWidth ?? this.properties.width, options.resizeHeight ?? this.properties.height] as const
            : undefined

        const newImageData = OpWrappers.opImageTranscode({
            data: this.data,
            inFormat: this.properties.formatName,
            outFormat: options.outputFormat,
            resize,
        });

        const newImageProperties: ImageProperties = {
            ...this.properties,
        }

        if (resize) {
            newImageProperties.width = resize[0]
            newImageProperties.height = resize[1]
        }

        return new Image(newImageData, newImageProperties)
    }

    dataUri(): string {
        const imageDataEncoded = base64Encode(this.data)
        const mimeType = formatMimeType(this.properties.formatName)

        return "data:" + mimeType + ";base64," + imageDataEncoded
    }
}

export type ImageTranscodeOptions = {
    outputFormat: SupportedEncodeImageFormat,
    resizeHeight?: number
    resizeWidth?: number
}

function isEncodable(format: SupportedImageFormat): format is SupportedEncodeImageFormat {
    return format === "png" || format === "webp" || format === "jpeg" || format === "gif"
}

export interface ImageProperties {
    formatName: SupportedImageFormat
    width: number
    height: number
}

function formatMimeType(format: SupportedImageFormat): string {
    switch (format) {
        case "png":
            return "image/png"
        case "jpeg":
            return "image/jpeg"
        case "gif":
            return "image/gif"
        case "webp":
            return "image/webp"
        case "pnm":
            return "image/pnm"
        case "tiff":
            return "image/tiff"
        case "tga":
            return "image/tga"
        case "bmp":
            return "image/bmp"
        case "ico":
            return "image/ico"
        case "hdr":
            return "image/hdr"
        case "openexr":
            return "image/openexr"
        case "qoi":
            return "image/qoi"
        case "avif":
            return "image/avif"
    }
}
