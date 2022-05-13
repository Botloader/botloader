import { PermissionsError } from './error';
import { IPermissionOverwrite, PermissionOverwriteType } from '../generated/discord/index';
import { getCurrentGuildId } from './dapi';

export type PermissionResolvable = string | number | bigint | Permissions;

const Flags: [string, bigint][] = [
    ['CreateInstantInvite', 1n << 0n],
    ['KickMembers', 1n << 1n],
    ['BanMembers', 1n << 2n],
    ['Administrator', 1n << 3n],
    ['ManageChannels', 1n << 4n],
    ['ManageGuild', 1n << 5n],
    ['AddReactions', 1n << 6n],
    ['ViewAuditLog', 1n << 7n],
    ['PrioritySpeaker', 1n << 8n],
    ['Stream', 1n << 9n],
    ['ViewChannel', 1n << 10n],
    ['SendMessages', 1n << 11n],
    ['SendTtsMessages', 1n << 12n],
    ['ManageMessages', 1n << 13n],
    ['EmbedLinks', 1n << 14n],
    ['AttachFiles', 1n << 15n],
    ['ReadMessageHistory', 1n << 16n],
    ['MentionEveryone', 1n << 17n],
    ['UseExternalEmojis', 1n << 18n],
    ['ViewGuildInsights', 1n << 19n],
    ['Connect', 1n << 20n],
    ['Speak', 1n << 21n],
    ['MuteMembers', 1n << 22n],
    ['DeafenMembers', 1n << 23n],
    ['MoveMembers', 1n << 24n],
    ['UseVAD', 1n << 25n],
    ['ChangeNickname', 1n << 26n],
    ['ManageNicknames', 1n << 27n],
    ['ManageRoles', 1n << 28n],
    ['ManageWebhooks', 1n << 29n],
    ['ManageEmojisAndStickers', 1n << 30n],
    ['UseApplicationCommands', 1n << 31n],
    ['RequestToSpeak', 1n << 32n],
    ['ManageEvents', 1n << 33n],
    ['ManageThreads', 1n << 34n],
    ['CreatePublicThreads', 1n << 35n],
    ['CreatePrivateThreads', 1n << 36n],
    ['UseExternalStickers', 1n << 37n],
    ['SendMessagesInThreads', 1n << 38n],
    ['UseEmbeddedActivities', 1n << 39n],
    ['ModerateMembers', 1n << 40n]
]

/**
 * A utility class for interacting with Discord server permissions.
 * This is essentially a wrapper around BigInt.
 * 
 * Note: each instance is immutable and can't be changed, add/remove operations etc
 * does not change the current instance but instead returns a new instance with the changes applied
 */
export class Permissions {
    static readonly CreateInstantInvite = new Permissions(1n << 0n);
    static readonly KickMembers = new Permissions(1n << 1n);
    static readonly BanMembers = new Permissions(1n << 2n);
    static readonly Administrator = new Permissions(1n << 3n);
    static readonly ManageChannels = new Permissions(1n << 4n);
    static readonly ManageGuild = new Permissions(1n << 5n);
    static readonly AddReactions = new Permissions(1n << 6n);
    static readonly ViewAuditLog = new Permissions(1n << 7n);
    static readonly PrioritySpeaker = new Permissions(1n << 8n);
    static readonly Stream = new Permissions(1n << 9n);
    static readonly ViewChannel = new Permissions(1n << 10n);
    static readonly SendMessages = new Permissions(1n << 11n);
    static readonly SendTtsMessages = new Permissions(1n << 12n);
    static readonly ManageMessages = new Permissions(1n << 13n);
    static readonly EmbedLinks = new Permissions(1n << 14n);
    static readonly AttachFiles = new Permissions(1n << 15n);
    static readonly ReadMessageHistory = new Permissions(1n << 16n);
    static readonly MentionEveryone = new Permissions(1n << 17n);
    static readonly UseExternalEmojis = new Permissions(1n << 18n);
    static readonly ViewGuildInsights = new Permissions(1n << 19n);
    static readonly Connect = new Permissions(1n << 20n);
    static readonly Speak = new Permissions(1n << 21n);
    static readonly MuteMembers = new Permissions(1n << 22n);
    static readonly DeafenMembers = new Permissions(1n << 23n);
    static readonly MoveMembers = new Permissions(1n << 24n);
    static readonly UseVAD = new Permissions(1n << 25n);
    static readonly ChangeNickname = new Permissions(1n << 26n);
    static readonly ManageNicknames = new Permissions(1n << 27n);
    static readonly ManageRoles = new Permissions(1n << 28n);
    static readonly ManageWebhooks = new Permissions(1n << 29n);
    static readonly ManageEmojisAndStickers = new Permissions(1n << 30n);
    static readonly UseApplicationCommands = new Permissions(1n << 31n);
    static readonly RequestToSpeak = new Permissions(1n << 32n);
    static readonly ManageEvents = new Permissions(1n << 33n);
    static readonly ManageThreads = new Permissions(1n << 34n);
    static readonly CreatePublicThreads = new Permissions(1n << 35n);
    static readonly CreatePrivateThreads = new Permissions(1n << 36n);
    static readonly UseExternalStickers = new Permissions(1n << 37n);
    static readonly SendMessagesInThreads = new Permissions(1n << 38n);
    static readonly UseEmbeddedActivities = new Permissions(1n << 39n);
    static readonly ModerateMembers = new Permissions(1n << 40n);

    readonly value: bigint;

    constructor(...data: PermissionResolvable[]) {
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
    static resolve(...data: PermissionResolvable[]): bigint {
        let result = 0n;
        for (let v of data) {
            if (typeof v === "object") {
                result |= v.value;
            } else {
                result |= BigInt(v);
            }
        }

        return result;
    }

    /**
     * @param perms The permissions to check for.
     * @returns True if the current value has any of the given permissions.
     */
    hasAny(...perms: PermissionResolvable[]): boolean {
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
    hasAll(...perms: PermissionResolvable[]): boolean {
        for (let p of perms) {
            const result = Permissions.resolve(p);
            if ((this.value & result) !== result) return false;
        }
        return true;
    }

    /**
     * Returns a new set of permissions with the provided permissions added on to the current ones
     * @param perms The permissions to add.
     * @returns The resulting permissions.
     */
    add(...perms: PermissionResolvable[]): Permissions {
        return new Permissions(this.value | Permissions.resolve(...perms));
    }

    /**
     * Returns a new set of permissions with the provided permissions removed
     * @param perms The permissions to remove.
     * @returns The resulting permissions.
     */
    remove(...perms: PermissionResolvable[]): Permissions {
        return new Permissions(this.value & ~Permissions.resolve(...perms));
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

/**
 * This is a helper class to make creating permission overwrites easier.
 * 
 * @example ```ts
 * // create a member overwrite that targets a single member
 * let member_overwrite = PermissionOverwrite.member("123", new Permissions(Permissions.CreateInstantInvite, Permissions.SendMessages), new Permissions())
 * 
 * // create a role overwrite that targets a role
 * let role_overwrite = PermissionOverwrite.role("123", new Permissions(Permissions.CreateInstantInvite, Permissions.SendMessages), new Permissions())
 *
 * // create a role overwrite for the everyone role that targets everyone
 * let everyone_overwrite = PermissionOverwrite.everyone(new Permissions(Permissions.CreateInstantInvite, Permissions.SendMessages), new Permissions())
 *  ```
 */
export class PermissionOverwrite implements IPermissionOverwrite {
    allowRaw: string;
    denyRaw: string;
    kind: PermissionOverwriteType;
    id: string;

    constructor(kind: PermissionOverwriteType, id: string, allow: PermissionResolvable, deny: PermissionResolvable) {
        this.kind = kind;
        this.id = id;
        this.allowRaw = Permissions.resolve(allow).toString();
        this.denyRaw = Permissions.resolve(deny).toString();
    }

    static member(id: string, allow: PermissionResolvable, deny: PermissionResolvable) {
        return new PermissionOverwrite("Member", id, allow, deny);
    }

    static role(id: string, allow: PermissionResolvable, deny: PermissionResolvable) {
        return new PermissionOverwrite("Role", id, allow, deny);
    }

    static everyone(allow: PermissionResolvable, deny: PermissionResolvable) {
        return PermissionOverwrite.role(getCurrentGuildId(), allow, deny);
    }
}