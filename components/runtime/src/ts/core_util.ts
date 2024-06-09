import { OpWrappers } from "./op_wrappers";
import * as Internal from "./generated/internal/index";

const non_json = ["boolean", "number", "string"];

export namespace console {
    export function log(...args: any[]) {
        console.output({
            items: args,
            level: "log",
            skipCallers: 1,
        })
    }

    export function warn(...args: any[]) {
        console.output({
            items: args,
            level: "warn",
            skipCallers: 1,
        })
    }

    export function error(...args: any[]) {
        console.output({
            items: args,
            level: "error",
            skipCallers: 1,
        })
    }

    /**
     * Lower level output function providing more controls than the simpler log/warn/error functions
     * 
     * @internal
     */
    export function output({ level, items, includeCaller, skipCallers }: ConsoleOutputOptions) {
        let output = "";
        let first = true;
        for (let arg of items) {
            if (!first) {
                output += ", ";
            }
            first = false

            if (non_json.includes(typeof arg)) {
                output += arg;
            } else {
                output += JSON.stringify(arg);
            }
        }

        const skip = 2 + (skipCallers ?? 0)
        let [file, line, col] = (includeCaller ?? true)
            ? getCaller(skip)
            : [undefined, undefined, undefined];

        OpWrappers.consoleLog({
            message: output,
            fileName: file,
            lineNumber: line,
            colNumber: col,
            level,
        })
    }
}

export interface ConsoleOutputOptions {
    /**
     * The items to log, they will be serialized to json if they're not a string
     */
    items: any[],

    level: Internal.ConsoleLogLevel,

    /**
     * @internal
     * 
     * Wheter to include the caller script, line and column number
     * 
     * Defaults to `true` if not provided
     * 
     * Marked as internal as im not sure if there is any use cases for this outside the botloader sdk
     */
    includeCaller?: boolean,

    /**
     * @internal
     * The number of callers to skip in the stack trace
     */
    skipCallers?: number,

    /**
     * @internal
     * 
     * Custom filename, line and column that will should in the log message prefix
     * 
     * Marked as internal as im not sure if there is any use cases for this outside the botloader sdk
     */
    customFileLineCol?: { file: string, line: string, col: string }
}

// after the many hours i've spent digging around in v8, i still don't know the proper way of getting a stack trace.
// so here's a hacky solution for now
function getCaller(skip: number) {
    const stack = (new Error()).stack;
    return parseStackEntry(skip, stack || "")
}

/** 
 * Returns a single line of a stack trace
 * 
 * @internal
 */
export function parseStackEntry(skip: number, stack: string) {
    const lines = stack!.split("\n");

    // get the correct line
    let selectedLine: string;
    if (skip >= lines.length - 2) {
        selectedLine = lines[lines.length - 1];
    } else {
        selectedLine = lines[skip + 1];
    }

    // parse it
    const re = /(file:\/\/\/.+):(\d+):(\d+)/
    const match = selectedLine.match(re);
    if (!match || match.length < 4) {
        return [undefined, undefined, undefined] as const
    }

    return [match[1], parseInt(match[2]), parseInt(match[3])] as const
}

(globalThis as any).console = {
    log: console.log,
    warn: console.warn,
    error: console.error,
    output: console.output,
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

/**
 * An async lock only runs 1 function at a time, this is useful if you have a series of async operations that depend on the computation of previous operation.
 * 
 * @example An example could be changing a single field in a json object:
 * 
 * ```ts
 * let storageBucket = script.createStorageVarJson<{ a: number, b: number }>("test_async_lock")
 * storageBucket.set({ a: 0, b: 0 })
 * 
 * async function badMutateField() {
 *     // To change a single field we first need to retrieve the whole object
 *     let currentValue = (await storageBucket.get())!.value
 * 
 *     // Then we save it with a new value for the 'a' field
 *     await storageBucket.set({ ...current, a: currentValue.a + 10 })
 * 
 *     // But what happens if you run this function twice at the same time? Or if its changed somewhere else in the meantime?
 *     // well it will be pretty random what happens then, it might sometimes work and it might sometimes not
 * }
 * 
 * // In fact just running it twice like this without awaiting (causing both promises to run in the background at the same time):
 * badMutateField()
 * badMutateField()
 * // Would create a bad result
 * 
 * // To fix this we create a async lock
 * const lock = new AsyncLock()
 * 
 * async function goodMutateField() {
 *     // Then we use the withLocked method and give it a function to run
 *     //
 *     // This will produce the expected result of `a` being 20
 *     // This is because AsyncLock only runs 1 function at a time
 *     lock.withLocked(badMutateField)
 *     lock.withLocked(badMutateField)
 * } 
 * ```
 */
export class AsyncLock {
    private locked = false
    private waiting: ((arg: any) => any)[] = []

    private async lock() {
        if (this.locked) {
            await new Promise((complete) => {
                if (!this.locked) {
                    this.locked = true
                    complete(null)
                } else {
                    this.waiting.push(complete)
                }
            })
        } else {
            this.locked = true
        }
    }

    private unlock() {
        let next = this.waiting.shift()
        if (next) {
            next(null)
        } else {
            this.locked = false
        }
    }

    /**
     * Run the provided function exclusively on the lock
     */
    async withLocked<T>(cb: () => T): Promise<Awaited<T>> {
        await this.lock()

        try {
            return await cb()
        } finally {
            this.unlock()
        }
    }
}