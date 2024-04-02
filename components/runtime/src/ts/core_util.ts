import { OpWrappers } from "./op_wrappers";

const non_json = ["boolean", "number", "string"];

/** 
 * @deprecated console is now a global, please remove the import
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