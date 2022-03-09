declare module 'js-untar' {
    export interface File {
        name: string,
        mode: string,
        type: string,

        readAsString(): string,
        readAsJSON(): unknown,
    }

    export default function untar(buf: ArrayBuffer): Promise<File[]>;
}