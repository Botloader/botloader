import { PermissionsError } from './error';

export type PermissionResolvable = (string | number | bigint)[];

const Flags: [string, bigint][] = [
    ['CreateInstantInvite',       1n << 0n],
    ['KickMembers',               1n << 1n],
    ['BanMembers',                1n << 2n],
    ['Administrator',             1n << 3n],
    ['ManageChannels',            1n << 4n],
    ['ManageGuild',               1n << 5n],
    ['AddReactions',              1n << 6n],
    ['ViewAuditLog',              1n << 7n],
    ['PrioritySpeaker',           1n << 8n],
    ['Stream',                    1n << 9n],
    ['ViewChannel',               1n << 10n],
    ['SendMessages',              1n << 11n],
    ['SendTtsMessages',           1n << 12n],
    ['ManageMessages',            1n << 13n],
    ['EmbedLinks',                1n << 14n],
    ['AttachFiles',               1n << 15n],
    ['ReadMessageHistory',        1n << 16n],
    ['MentionEveryone',           1n << 17n],
    ['UseExternalEmojis',         1n << 18n],
    ['ViewGuildInsights',         1n << 19n],
    ['Connect',                   1n << 20n],
    ['Speak',                     1n << 21n],
    ['MuteMembers',               1n << 22n],
    ['DeafenMembers',             1n << 23n],
    ['MoveMembers',               1n << 24n],
    ['UseVAD',                    1n << 25n],
    ['ChangeNickname',            1n << 26n],
    ['ManageNicknames',           1n << 27n],
    ['ManageRoles',               1n << 28n],
    ['ManageWebhooks',            1n << 29n],
    ['ManageEmojisAndStickers',   1n << 30n],
    ['UseApplicationCommands',    1n << 31n],
    ['RequestToSpeak',            1n << 32n],
    ['ManageEvents',              1n << 33n],
    ['ManageThreads',             1n << 34n],
    ['CreatePublicThreads',       1n << 35n],
    ['CreatePrivateThreads',      1n << 36n],
    ['UseExternalStickers',       1n << 37n],
    ['SendMessagesInThreads',     1n << 38n],
    ['UseEmbeddedActivities',     1n << 39n],
    ['ModerateMembers',           1n << 40n]
]

/**
 * A utility class for interacting with Discord server permissions.
 * This is essentially a wrapper around BigInt.
 */
export class Permissions {
    static CreateInstantInvite      = 1n << 0n;
    static KickMembers              = 1n << 1n;
    static BanMembers               = 1n << 2n;
    static Administrator            = 1n << 3n;
    static ManageChannels           = 1n << 4n;
    static ManageGuild              = 1n << 5n;
    static AddReactions             = 1n << 6n;
    static ViewAuditLog             = 1n << 7n;
    static PrioritySpeaker          = 1n << 8n;
    static Stream                   = 1n << 9n;
    static ViewChannel              = 1n << 10n;
    static SendMessages             = 1n << 11n;
    static SendTtsMessages          = 1n << 12n;
    static ManageMessages           = 1n << 13n;
    static EmbedLinks               = 1n << 14n;
    static AttachFiles              = 1n << 15n;
    static ReadMessageHistory       = 1n << 16n;
    static MentionEveryone          = 1n << 17n;
    static UseExternalEmojis        = 1n << 18n;
    static ViewGuildInsights        = 1n << 19n;
    static Connect                  = 1n << 20n;
    static Speak                    = 1n << 21n;
    static MuteMembers              = 1n << 22n;
    static DeafenMembers            = 1n << 23n;
    static MoveMembers              = 1n << 24n;
    static UseVAD                   = 1n << 25n;
    static ChangeNickname           = 1n << 26n;
    static ManageNicknames          = 1n << 27n;
    static ManageRoles              = 1n << 28n;
    static ManageWebhooks           = 1n << 29n;
    static ManageEmojisAndStickers  = 1n << 30n;
    static UseApplicationCommands   = 1n << 31n;
    static RequestToSpeak           = 1n << 32n;
    static ManageEvents             = 1n << 33n;
    static ManageThreads            = 1n << 34n;
    static CreatePublicThreads      = 1n << 35n;
    static CreatePrivateThreads     = 1n << 36n;
    static UseExternalStickers      = 1n << 37n;
    static SendMessagesInThreads    = 1n << 38n;
    static UseEmbeddedActivities    = 1n << 39n;
    static ModerateMembers          = 1n << 40n;

    value: bigint;

    constructor(...data: PermissionResolvable) {
        this.value = Permissions.resolve(...data);
    }

    /**
     * @returns All the Discord permissions as a single bit.
     */
    static get rawAll(): bigint {
        return Flags.reduce((a, b) => a | b[1], 0n);
    }

    /**
     * @returns An object containing all Discord permissions.
     */
    static entries(): { [key: string]: bigint } {
        return Flags.reduce<{ [key: string]: bigint }>((a, b) => {
            a[b[0]] = b[1];
            return a;
        }, {});
    }

    /**
     * A static method for resolving permissions from strings, numbers and bigints.
     * @param data The data to resolve permissions from.
     * @returns The resolved bits.
     */
    static resolve(...data: PermissionResolvable): bigint {
        let result = 0n;
        const entries = this.entries();
        for (let i of data) {
            switch (typeof i) {
                case 'string':
                    if (entries[i] !== undefined) {
                        result |= entries[i];
                    } else {
                        throw new PermissionsError(`Unknown permission '${i}'`);
                    }
                    break;
                case 'number': result |= BigInt(i); break;
                default: result |= i; break;
            }
        }
        return result;
    }

    /**
     * @param perms The permissions to check for.
     * @returns True if the current value has any of the given permissions.
     */
    hasAny(...perms: PermissionResolvable): boolean {
        for (let p of perms) {
            const result = Permissions.resolve(p);
            if ((this.value & result) === result) return true;
        }
        return false;
    }

    /**
     * @param perms The permissions to check for.
     * @returns True if the current value has all of the given permissions.
     */
    hasAll(...perms: PermissionResolvable): boolean {
        for (let p of perms) {
            const result = Permissions.resolve(p);
            if ((this.value & result) !== result) return false;
        }
        return true;
    }

    /**
     * Adds the given permissions to the current value.
     * @param perms The permissions to add.
     * @returns The resulting permissions.
     */
    add(...perms: PermissionResolvable): bigint {
        this.value |= Permissions.resolve(...perms);
        return this.value;
    }

    /**
     * Removes the given permissions from the current value.
     * @param perms The permissions to remove.
     * @returns The resulting permissions.
     */
    remove(...perms: PermissionResolvable): bigint {
        this.value &= ~Permissions.resolve(...perms);
        return this.value;
    }

    /**
     * @returns The current permissions value as an array of strings.
     */
    toArray(): string[] {
        return Flags
            .filter(f => this.hasAny(f[1]))
            .map(f => f[0]);
    }

    /**
     * @returns The string value of the permissions.
     */
    toString(): string {
        return this.value.toString();
    }
}
