export interface SnowflakeData {
    id:         bigint;
    timestamp:  bigint;
    worker:     bigint;
    process:    bigint;
    increment:  bigint;
}

/**
 * A utility class for constructing and deconstructing Discord IDs,
 * also known as snowflakes.
 */
export class Snowflake {
    static readonly EPOCH = 1420070400000n;

    private constructor() {}

    /**
     * Constructs a unique Discord snowflake.
     * @see {@link SnowflakeData}
     * @param [options] Options to apply to the snowflake.
     * @returns The constucted snowflake.
     */
    static construct(options: Partial<SnowflakeData> = {}): bigint {
        const timestamp = BigInt(options?.timestamp ?? Date.now());
        const increment = options.increment
            ? options.increment >= 4095n
                ? 0n
                : options.increment
            : 0n;

        return ((timestamp - this.EPOCH) << 22n)
            | (options.worker ?? 0n & 0b11111n) << 17n
            | (options.process ?? 1n & 0b11111n) << 12n
            | increment;
    }

    /**
     * Deconstructs a snowflake into its components.
     * @see {@link SnowflakeData}
     * @param id The snowflake to deconstruct.
     * @returns The deconstructed snowflake.
     */
    static deconstruct(id: string | bigint): SnowflakeData {
        id = BigInt(id);
        return {
            id,
            timestamp: (id >> 22n) + this.EPOCH,
            worker: (id >> 17n) & 0b11111n,
            process: (id >> 12n) & 0b11111n,
            increment: id & 0b111111111111n
        }
    }
}
