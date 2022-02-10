import { Ops } from "./models";
import { OpWrappers } from "./op_wrappers";

/**
 * The storage namespace contains API's for persistent storage within botloader.
 */
export namespace Storage {


    export interface SetValueOptions {
        /**
         * Optional time to live in seconds for the value, after which it is deleted. 
         */
        ttl?: number,
    }

    export interface ListOptions {
        /**
         * Only return entires after this key.
         * 
         * Use this to paginate through results, using the key of the last entry in the list call. 
         */
        after?: string,

        /**
         * Number of entries to return, max 100.
         * 
         * Defaults to 25 as of writing.
         */
        limit?: number,

        /**
         * Only return entries that match the pattern,
         * 
         *  - `%`: A percent sign (%) matches any sequence of zero or more characters.
         *  - `_`: An underscore (_) matches any single character.
         * 
         * To match _ and % as literals escape them with backslash (`\_` and `\%`).
         * 
         * @example
         * `user\_%` will match `user_123123`
         */
        keyPattern?: string
    }

    export interface SortedListOptions {
        /**
         * How many entries to skip, useful for paginating through the list
         */
        offset?: number,

        /**
         * Number of entries to return, max 100.
         * 
         * Defaults to 25 as of writing.
         */
        limit?: number,
    }

    export interface Entry<T> {
        /**
         * The bucket this entry was in
         */
        bucket: string,
        /**
         * Key where this entry was stored at the time of fetching
         */
        key: string,
        /**
         * Value this entry holds
         */
        value: T,

        /**
         * If a ttl was set, when this entry expires
         */
        expiresAt?: Date,
    }

    /**
     * 
     * A Bucket provides persistent storage to botloader, using this you can store data and have it persist across vm reloads and bot restarts.
     * 
     * Buckets are namespaces, A Bucket with the name `a` holds different values from another Bucket with the name `b` even though the keys might be the same.
     * 
     * @remark this bucket should be registered with your script or plugin (example: `script.registerStorageBucket(...)`).
     * 
     * @typeParam T - The type of values stored in this bucket
     */
    export abstract class Bucket<T>{
        name: string;

        /**
         * Create a new storage bucket.
         * 
         * @remark this bucket should be registered with your script or plugin (example: `script.registerStorageBucket(...)`),.
         * 
         * @param name The name of the bucket. Note that this is not unique across your scripts and
         * the same name in one script will have the same values as in another and is perfectly safe.
         */
        constructor(name: string) {
            this.name = name;
        }


        protected abstract intoInternalValue(v: T): Ops.OpStorageBucketValue;
        protected abstract fromInternalValue(v: Ops.OpStorageBucketValue): T | undefined;

        protected entryFromInternal(entry: Ops.OpStorageBucketEntry): Entry<T> {
            let val = this.fromInternalValue(entry.value);
            if (val == undefined) {
                throw new Error("failed converting from internal value, incorrect type. This can happen if you changed the bucket type from say number to json, don't do that.");
            }

            return {
                bucket: this.name,
                key: entry.key,
                value: val,
                expiresAt: entry.expiresAt ? new Date(entry.expiresAt) : undefined,
            }
        }

        protected entryFromInternalOptional(entry?: Ops.OpStorageBucketEntry | null): Entry<T> | undefined {
            if (entry) {
                return this.entryFromInternal(entry)
            } else {
                return undefined
            }
        }

        /**
         * Store a value at the provided key in the bucket, this will overwrite the previous value stored there, if any.
         * 
         * @param key The key that you're storing the value in 
         * @param value The value you're storing, for objects this will be converted to json behind the scenes
         * @param options Optional options
         * @returns The storage entry
         */
        async set(key: string, value: T, options?: SetValueOptions) {
            return this.entryFromInternal(await OpWrappers.bucketStorageSet({
                bucketName: this.name,
                key: key,
                value: this.intoInternalValue(value),
                ttl: options?.ttl,
            }));
        }

        /**
         * Similar to {@link set} but stores the value conditionally.
         * 
         * @param key The key where you're storing the value
         * @param value The value you're storing, for objects this will be converted to json behind the scenes
         * @param cond The condition that has to pass to store the value.
         *  - IfExists: will only store the value if the key existed beforehand. 
         *  - IfNotExists: will only store the value if the key did not exist. 
         * @param options Optional options
         * @returns Either the new entry, or undefined if the condition failed. 
         */
        async setIf(key: string, value: T, cond: "IfExists" | "IfNotExists", options?: SetValueOptions) {
            return this.entryFromInternalOptional(await OpWrappers.bucketStorageSetIf({
                bucketName: this.name,
                key,
                value: this.intoInternalValue(value),
                ttl: options?.ttl,
                cond,
            }));
        }

        /**
         * Fetches a entry from the bucket.
         * 
         * @param key The key for the value you want returned
         * @returns The entry, or undefined if it did not exist
         */
        async get(key: string) {
            return this.entryFromInternalOptional(await OpWrappers.bucketStorageGet({
                bucketName: this.name,
                key: key,
            }));
        }

        /**
         * Deletes an entry from the bucket permanently.
         * 
         * @param key The key to delete
         * @returns The deleted entry, or undefined if none
         */
        async del(key: string) {
            return this.entryFromInternalOptional(await OpWrappers.bucketStorageDel({
                bucketName: this.name,
                key: key,
            }));
        }

        /**
         * Retrieve a list of entries from the database, you can use `after` to paginate through all the items in the bucket.
         * 
         * @param options Pagination options
         * @returns A list of entries
         */
        async list(options: ListOptions) {
            const res = await OpWrappers.bucketStorageList({
                bucketName: this.name,
                after: options.after,
                keyPattern: options.keyPattern,
                limit: options.limit,
            });

            return res.map(v => this.entryFromInternal(v));
        }
    }

    /**
     * A Bucket holding number values
     * 
     * The values being numbers allows them to be sorted easily giving you access to {@link incr} and {@link sortedList}.
     * 
     * {@see} {@link Bucket} for more info on buckets.
     */
    export class NumberBucket extends Bucket<number>{
        protected intoInternalValue(v: number): Ops.OpStorageBucketValue {
            return {
                double: v,
            }
        }

        protected fromInternalValue(v: Ops.OpStorageBucketValue): number | undefined {
            if ('double' in v) {
                return v.double;
            }

            return undefined;
        }

        /**
         * Atomically increments the value stored at key. If the entry did not exist beforehand a new one is created and set to `amount`
         * 
         * @param key The key whose value to increment
         * @param amount The amount to increment the value by
         * @returns The entry after incrementing the value
         */
        async incr(key: string, amount: number) {
            return this.entryFromInternal(await OpWrappers.bucketStorageIncr({
                bucketName: this.name,
                key: key,
                amount: amount,
            }));
        }

        /**
         * Returns a list of entries in the bucket sorted by values
         * 
         * @param order The order of which you want the entries in the bucket sorted by
         * @param options Pagination options
         */
        async sortedList(order: "Ascending" | "Descending", options?: SortedListOptions) {
            const res = await OpWrappers.bucketStorageSortedList({
                bucketName: this.name,
                limit: options?.limit,
                offset: options?.offset,
                order: order,
            });

            return res.map(v => this.entryFromInternal(v));
        }
    }

    /**
     * A Bucket holding json objects
     * 
     * {@see} {@link Bucket} for more info on buckets.
     */
    export class JsonBucket<T> extends Bucket<T>{
        protected intoInternalValue(v: T): Ops.OpStorageBucketValue {
            return {
                // json is handled on the rust side and opcall side
                json: v,
            }
        }

        protected fromInternalValue(v: Ops.OpStorageBucketValue): T | undefined {
            if ('json' in v) {
                // json is handled on the rust side and opcall side
                return v.json;
            }

            return undefined
        }
    }
}