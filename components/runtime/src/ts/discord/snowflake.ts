export const DiscordEpoch = 1420070400000n;

/**
 * Gets the timestamp from a Discord ID.
 * @param id The snowflake to deconstruct.
 * @returns The snowflake timestamp.
 */
export function snowflakeTimestamp(id: string): number {
    return Number((BigInt(id) >> 22n) + DiscordEpoch);
}
