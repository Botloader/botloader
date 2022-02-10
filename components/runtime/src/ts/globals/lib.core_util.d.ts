import { console as _console } from '../core_util';

declare global {
    const console: {
        log: typeof _console.log,
    }
} 