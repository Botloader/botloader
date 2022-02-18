import { OpWrappers } from "./op_wrappers";

const non_json = ["boolean", "number", "string"];

/** * @deprecated console is now a global, please remove the import
 */
export namespace console {
    export function log(...args: any[]) {
        let output = "";
        const first = true;
        for (let arg of args) {
            if (!first) {
                output += ", ";
            }

            if (non_json.includes(typeof arg)) {
                output += arg;
            } else {
                output += JSON.stringify(arg);
            }
        }

        let [file, line, col] = getCaller(2);

        OpWrappers.consoleLog({
            message: output,
            fileName: file,
            lineNumber: line,
            colNumber: col,
        })
    }
}

// after the many hours i've spent digging around in v8, i still don't know the proper way of getting a stack trace.
// so here's a hacky solution for now
function getCaller(skip: number): [string | undefined, number | undefined, number | undefined] {
    const stack = (new Error()).stack;
    const lines = stack!.split("\n");

    // get the correct line
    let selected_line: string;
    if (skip >= lines.length - 2) {
        selected_line = lines[lines.length - 1];
    } else {
        selected_line = lines[skip + 1];
    }

    // parse it
    const re = /(file:\/\/\/.+):(\d+):(\d+)/
    const match = selected_line.match(re);
    if (!match || match.length < 4) {
        return [undefined, undefined, undefined]
    }

    return [match[1], parseInt(match[2]), parseInt(match[3])]
}

(globalThis as any).console = {
    log: console.log,
};

/**
 * Encode a string to its Uint8Array representation.
 */
export function encodeText(s: string): Uint8Array {
    return Deno.core.encode(s);
}

/**
 * Decode a string from its Uint8Array representation.
 */
export function decodeText(buf: Uint8Array): string {
    return Deno.core.decode(buf);
}