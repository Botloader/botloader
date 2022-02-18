export interface AsyncWriter {
    write(data: Uint8Array): Promise<number>;
    close(): void;
}

export interface AsyncReader {
    read(buf: Uint8Array): Promise<number>;
}

export interface AsyncReadCloser extends AsyncReader {
    close(): void;
}

/**
 * @internal
 */
export class NativeReaderWriter implements AsyncReader, AsyncWriter, AsyncReadCloser {
    rid: number;

    constructor(rid: number) {
        this.rid = rid
    }

    read(buf: Uint8Array): Promise<number> {
        return Deno.core.read(this.rid, buf);
    }

    write(data: Uint8Array): Promise<number> {
        return Deno.core.write(this.rid, data)
    }

    close(): void {
        Deno.core.close(this.rid);
    }
}

/**
 * @internal
 */
export class NativeReader implements AsyncReader, AsyncReadCloser {
    rid: number;

    constructor(rid: number) {
        this.rid = rid
    }

    read(buf: Uint8Array): Promise<number> {
        return Deno.core.read(this.rid, buf);
    }

    close(): void {
        Deno.core.close(this.rid);
    }
}