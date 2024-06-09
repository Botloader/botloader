import { console as _console } from '../core_util';

declare global {
    const console: {
        log: typeof _console.log,
        error: typeof _console.error,
        warn: typeof _console.warn,
        output: typeof _console.output,
    }
}