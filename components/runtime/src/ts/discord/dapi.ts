import { Guild, Role, Embed, IComponent, AuditLogExtras, SendEmoji, IPermissionOverwrite, VideoQualityMode, ChannelType, PermissionOverwriteType, InviteTargetType } from '../generated/discord/index';
import * as Internal from '../generated/internal/index';
import { OpWrappers } from '../op_wrappers';
import { GuildChannel, Thread, ThreadMember, guildChannelFromInternal, threadChannelFromInternal } from './channel';
import type { AutoArchiveMinutes } from './channel';
import { VoiceState } from './events';
import { Invite } from './invite';
import { Ban, Member } from './member';
import { Message } from './message';
import { Permissions } from './permissions';
import { User } from './user';

/**
 * @returns Botloader's discord user 
 */
export function getBotUser(): User {
    return new User(OpWrappers.getCurrentUser());
}

/**
 * @returns The current guild's Id
 */
export function getCurrentGuildId(): string {
    return OpWrappers.getCurrentGuildId();
}

// Guild functions
export function getGuild(): Promise<Guild> {
    return OpWrappers.callAsyncOp({
        kind: "discord_get_guild",
        arg: null,
    })
}

function editGuild() { }

/**
 * @returns A list of the invites on your server
 */
export async function getGuildInvites(): Promise<Invite[]> {
    return (await OpWrappers.getInvites()).map(v => new Invite(v))
}

export async function getInvite(code: string, options?: {
    withCounts?: boolean,
    withExpiration?: boolean,
}): Promise<Invite> {
    let result = await OpWrappers.getInvite(code, options?.withCounts ?? false, options?.withExpiration ?? false)
    return new Invite(result)
}

export async function deleteInvite(code: string): Promise<void> {
    await OpWrappers.deleteInvite(code)
}

// Message functions
export async function getMessage(channelId: string, messageId: string): Promise<Message> {
    return new Message(await OpWrappers.callAsyncOp({
        kind: "discord_get_message",
        arg: [channelId, messageId],
    }));
}

export interface GetMessagesOptions {
    /**
     * Limit max results, max 100, default 50
     */
    limit?: number,

    /**
     * Return messages made after this message id
     */
    after?: string,
    /**
     * Return messages made before this message id
     */
    before?: string,
}

export async function getMessages(channelId: string, options?: GetMessagesOptions): Promise<Message[]> {
    return (await OpWrappers.callAsyncOp({
        kind: "discord_get_messages",
        arg: {
            channelId,
            after: options?.after,
            before: options?.before,
            limit: options?.limit,
        }
    })).map(v => new Message(v));
}


export interface BaseCreateMessageFields {
    content?: string;
    embeds?: Embed[];

    /**
     * Control the mentions in the message.
     * 
     * The default for this if not provided is: {parse: ["users"]}
     * this means that discord will search the message for user mentions only and 'everyone', 'here' and other mentions
     * will be ignored.
     */
    allowedMentions?: AllowedMentions;

    components?: IComponent[],
}

export interface CreateMessageFields extends BaseCreateMessageFields {
    /**
     * Shows up as a reply to the following message
     */
    replyToMessageId?: string;
}

export interface InteractionCreateMessageFields extends BaseCreateMessageFields {
    flags?: InteractionMessageFlags,
}


export interface InteractionMessageFlags {
    /**
     * Ephemeral messages can only be seen by the author of the interaction
     */
    ephemeral?: boolean,

    suppressEmbeds?: boolean,
}


export interface AllowedMentions {
    /**
     * Types of mentions to parse from the message
     */
    parse: MentionParseTypes[];
    /**
     * Array of role_ids to mention (Max size of 100)
     */
    users?: string[];
    /**
     * Array of user_ids to mention (Max size of 100)
     */
    roles?: string[];

    /**
     * For replies, whether to mention the author of the message being replied to (default false)
     */
    repliedUser?: boolean;
}

/**
 * @internal
 */
export function toOpMessageFields(fields: CreateMessageFields): Internal.OpCreateMessageFields {
    let allowedMentions: Internal.AllowedMentions;
    if (fields.allowedMentions) {
        allowedMentions = {
            parse: fields.allowedMentions.parse,
            users: fields.allowedMentions.users ?? [],
            roles: fields.allowedMentions.roles ?? [],
            repliedUser: fields.allowedMentions.repliedUser ?? false,
        }
    } else {
        allowedMentions = {
            parse: ["Users"],
            users: [],
            roles: [],
            repliedUser: false,
        }
    }

    return {
        ...fields,
        allowedMentions: allowedMentions!,
    }
}

export type MentionParseTypes = "Everyone" | "Roles" | "Users";

export async function createMessage(channelId: string, fields: CreateMessageFields): Promise<Message> {

    return new Message(await OpWrappers.callAsyncOp({
        kind: "discord_create_message",
        arg: {
            channelId,
            fields: toOpMessageFields(fields),
        }
    }));
}
export async function editMessage(channelId: string, messageId: string, fields: CreateMessageFields): Promise<Message> {
    return new Message(await OpWrappers.callAsyncOp({
        kind: "discord_edit_message",
        arg: {
            channelId,
            messageId,
            fields: toOpMessageFields(fields),
        }
    }));
}

export async function crosspostMessage(channelId: string, messageId: string): Promise<void> {
    await OpWrappers.callAsyncOp({
        kind: "discord_crosspost_message",
        arg: [channelId, messageId]
    });
}

export async function deleteMessage(channelId: string, messageId: string): Promise<void> {
    await OpWrappers.callAsyncOp({
        kind: "discord_delete_message",
        arg: {
            channelId,
            messageId,
        }
    })
}

export async function bulkDeleteMessages(channelId: string, ...messageIds: string[]): Promise<void> {
    await OpWrappers.callAsyncOp({
        kind: "discord_bulk_delete_messages",
        arg: {
            channelId,
            messageIds,
        }
    })
}


// Role functions
export function getRole(roleId: string): Promise<Role> {
    return OpWrappers.getRole(roleId);
}
export function getRoles(): Promise<Role[]> {
    return OpWrappers.getRoles();
}

async function createRole() { }
async function editRole() { }
async function deleteRole() { }

// Channel functions
export async function getChannel(channelId: string): Promise<GuildChannel> {
    return guildChannelFromInternal(await OpWrappers.getChannel(channelId));
}
export async function getChannels(): Promise<GuildChannel[]> {
    return (await OpWrappers.getChannels()).map(v => guildChannelFromInternal(v));
}

export interface ICreateChannel {
    name: string;
    kind?: ChannelType;
    bitrate?: number;
    nsfw?: boolean;
    parentId?: string;

    /**
     * You can use the {@see PermissionOverwrite} class here. 
     * @example ```ts
     * {
     *      permissionOverwrites: [Discord.PermissionOverwrite.member("213", new Permissions(Permissions.CreateInstantInvite, Permissions.SendMessages), new Permissions()]
     * }
     *  ```
     */
    permissionOverwrites?: IPermissionOverwrite[];
    position?: number;
    rateLimitPerUser?: number;
    topic?: string;
    userLimit?: number;
}

export async function createChannel(fields: ICreateChannel): Promise<GuildChannel> {
    return guildChannelFromInternal(await OpWrappers.createChannel(fields));
}

/**
 * All fields are optional, fields you don't set will not be changed.
 */
export interface IEditChannel {
    bitrate?: number;
    name?: string;
    nsfw?: boolean;
    parentId?: string | null;

    /**
     * You can use the {@see PermissionOverwrite} class here. 
     * @example ```ts
     * {
     *      permissionOverwrites: [Discord.PermissionOverwrite.member("213", new Permissions(Permissions.CreateInstantInvite, Permissions.SendMessages), new Permissions()]
     * }
     *  ```
     */
    permissionOverwrites?: IPermissionOverwrite[];
    position?: number;
    rateLimitPerUser?: number;
    topic?: string;
    userLimit?: number;
    videoQualityMode?: VideoQualityMode;
}

export async function editChannel(channelId: string, fields: IEditChannel): Promise<GuildChannel> {
    return guildChannelFromInternal(await OpWrappers.editChannel(channelId, fields));
}

export interface EditChannelPosition {
    channelId: string,
    position: number,
}

/**
 * Edits the positions of multiple channels at the same time
 * 
 * Requires `MANAGE_CHANNELS` permission
 */
export async function editChannelPositions(channels: EditChannelPosition[]): Promise<void> {
    await OpWrappers.callAsyncOp({
        kind: "discord_bulk_edit_channels",
        arg: channels,
    })
}

export async function deleteChannel(channelId: string): Promise<GuildChannel> {
    return guildChannelFromInternal(await OpWrappers.deleteChannel(channelId));
}

export async function editChannelPermission(channelId: string, overwrite: IPermissionOverwrite): Promise<void> {
    return OpWrappers.updateChannelPermission(channelId, overwrite);
}

export async function deleteChannelPermission(channelId: string, kind: PermissionOverwriteType, id: string): Promise<void> {
    return OpWrappers.deleteChannelPermission(channelId, kind, id);
}

export async function getVoiceStates(): Promise<VoiceState[]> {
    return OpWrappers.getVoiceStates()
}

export async function getChannelInvites(channelId: string): Promise<Invite[]> {
    return (await OpWrappers.getChannelInvites(channelId)).map(v => new Invite(v))
}

export interface ICreateInviteFields {
    /**
     * Duration of invite in seconds before expiry, or 0 for never. between 0 and 604800 (7 days)
     * 
     * Default: 24 hours
     */
    maxAgeSeconds?: number;

    /**
     * Max number of uses or 0 for unlimited. between 0 and 100
     * 
     * Default: 0 (unlimited)
     */
    maxUses?: number;

    /**
     * Whether this invite only grants temporary membership
     * 
     * Default: false
     */
    temporary?: boolean;

    /**
     * If true, don't try to reuse a similar invite (useful for creating many unique one time use invites)
     * 
     * Default false
     */
    unique?: boolean;

    /**
     * The type of target for this voice channel invite
     */
    targetType?: InviteTargetType;

    /**
     * The id of the user whose stream to display for this invite, required if targetType is 'Stream', the user must be streaming in the channel
     */
    targetUserId?: string;

    /**
     * The id of the embedded application to open for this invite, required if targetType is 'EmbeddedApplication', the application must have the EMBEDDED flag
     */
    targetApplicationId?: string;
}

export async function createChannelInvite(channelId: string, fields: ICreateInviteFields): Promise<Invite> {
    const result = await OpWrappers.createChannelInvite(channelId, {
        max_age: fields.maxAgeSeconds,
        max_uses: fields.maxUses,
        temporary: fields.temporary,
        target_application_id: fields.targetApplicationId,
        target_user_id: fields.targetUserId,
        target_type: fields.targetType,
        unique: fields.unique,
    })

    return new Invite(result)
}

// Pins 
export async function getPins(channelId: string): Promise<Message[]> {
    return (await OpWrappers.op_discord_get_channel_pins(channelId)).map(v => new Message(v));
}
export async function createPin(channelId: string, messageId: string): Promise<void> {
    return OpWrappers.op_discord_create_pin(channelId, messageId);
}
export async function deletePin(channelId: string, messageId: string): Promise<void> {
    return OpWrappers.op_discord_delete_pin(channelId, messageId);
}

// Emoji functions
async function getEmoji() { }
async function getEmojis() { }
async function createEmoji() { }
async function editEmoji() { }
async function deleteEmoji() { }


// Sticker functions
async function getSticker() { }
async function getStickers() { }
async function createSticker() { }
async function editSticker() { }
async function deleteSticker() { }

export async function getMember(id: string): Promise<Member | undefined> {
    const member = (await OpWrappers.getMembers([id]))[0];
    if (member) {
        return new Member(member);
    }

    return undefined;
}

export async function getMembers(ids: string[]): Promise<(Member | null)[]> {
    return (await OpWrappers.getMembers(ids)).map(v => v ? new Member(v) : null);
}

/**
 * Fields that are not provided will be left unchanged.
 */
export interface UpdateGuildMemberFields {
    /**
     * Update the members voice channel, or set to null to kick them from their current vocie channel.
     */
    channelId?: string | null;


    deaf?: boolean;
    mute?: boolean;

    /**
     * Update the members nickname, or set to null to reset it
     */
    nick?: string | null;

    roles?: string[];

    /**
     * Updates the member's timeout duration, set to null to remove it.
     */
    communicationDisabledUntil?: number | null;
}

export async function editMember(userId: string, fields: UpdateGuildMemberFields): Promise<Member> {
    return new Member(await OpWrappers.updateMember(userId, fields));
}

export async function setMemberTimeout(userId: string, time: Date | null): Promise<Member> {
    return await editMember(userId, { communicationDisabledUntil: time ? time.getTime() : null });
}

export async function addMemberRole(userId: string, roleId: string): Promise<void> {
    return await OpWrappers.addMemberRole(userId, roleId);
}

export async function removeMemberRole(userId: string, roleId: string): Promise<void> {
    return await OpWrappers.removeMemberRole(userId, roleId);
}

export async function removeMember(userId: string, extras?: AuditLogExtras): Promise<void> {
    return OpWrappers.removeMember(userId, extras ?? {});
}


export async function getMemberGuildPermissions(member: Member): Promise<Permissions>;
export async function getMemberGuildPermissions(userId: string): Promise<Permissions>;

/**
 * Calculates the server permissions of a member
 * 
 * This function does not take channel overwrites into account, use {@see getMemberChannelPermissions} for that
 */
export async function getMemberGuildPermissions(memberOrUserId: Member | string): Promise<Permissions> {
    let userId = "";
    let memberRoles: string[] | null = null;
    if (typeof memberOrUserId === "string") {
        userId = memberOrUserId;
    } else {
        memberRoles = memberOrUserId.roles;
        userId = memberOrUserId.user.id;
    }

    let [guildPerms, _] = await OpWrappers.getMemberPermissions(userId, memberRoles, null);
    return new Permissions(guildPerms)
}

export interface CalculatedMemberPermissions {

    /**
     * Guild level permissions only
     */
    guild: Permissions,

    /**
     * Permissions in the channel
     * 
     * Note: only permissions relevant to channels are contained in this
     */
    channel: Permissions,

    /**
     * Channel id these perms were computed for
     */
    channelId: string,
}

export async function getMemberChannelPermissions(userId: string, channelId: string): Promise<CalculatedMemberPermissions>;
export async function getMemberChannelPermissions(member: Member, channelId: string): Promise<CalculatedMemberPermissions>;

/**
 * Calculates the server and channel permissions of a member
 * 
 */
export async function getMemberChannelPermissions(memberOrUserId: Member | string, channelId: string): Promise<CalculatedMemberPermissions> {
    let userId = "";
    let memberRoles: string[] | null = null;
    if (typeof memberOrUserId === "string") {
        userId = memberOrUserId;
    } else {
        memberRoles = memberOrUserId.roles;
        userId = memberOrUserId.user.id;
    }

    console.log("CHANNEL ID: ", channelId);
    let [guildPerms, channelPerms] = await OpWrappers.getMemberPermissions(userId, memberRoles, channelId);

    return {
        guild: new Permissions(guildPerms),
        channel: new Permissions(channelPerms ?? 0),
        channelId,
    };
}

// Guild bans
export interface CreateBanExtras extends AuditLogExtras {
    deleteMessageDays: number
}

export async function createBan(userId: string, extras?: CreateBanExtras): Promise<void> {
    return OpWrappers.createBan(userId, extras ?? {});
}

export async function getBan(userID: string): Promise<Ban> {
    return new Ban(await OpWrappers.getBan(userID));
}

export async function getBans(): Promise<Ban[]> {
    return (await OpWrappers.getBans()).map(v => new Ban(v));
}

export async function deleteBan(userId: string, extras?: AuditLogExtras): Promise<void> {
    return OpWrappers.removeBan(userId, extras ?? {});
}

// Reactions
export async function createReaction(channelId: string, messageId: string, emoji: SendEmoji): Promise<void> {
    return OpWrappers.discord_create_reaction(channelId, messageId, emoji);
}
export async function deleteOwnReaction(channelId: string, messageId: string, emoji: SendEmoji): Promise<void> {
    return OpWrappers.discord_delete_own_reaction(channelId, messageId, emoji);
}
export async function deleteUserReaction(channelId: string, messageId: string, userId: string, emoji: SendEmoji): Promise<void> {
    return OpWrappers.discord_delete_user_reaction(channelId, messageId, userId, emoji);
}

export interface GetReactionsExtras {
    /**
     * Return users after this Id.
     * You can use this to paginate through all the results.
     */
    after?: string,

    /**
     * Limit the number of results, defaults to 25, max 100 at the time of writing
     */
    limit?: number,
}

export async function getReactions(channelId: string, messageId: string, emoji: SendEmoji, extra?: GetReactionsExtras): Promise<User[]> {
    return (await OpWrappers.discord_get_reactions(channelId, messageId, {
        ...extra,
        emoji: emoji,
    })).map(v => new User(v));
}
export async function deleteAllReactions(channelId: string, messageId: string): Promise<void> {
    return OpWrappers.discord_delete_all_reactions(channelId, messageId);
}
export async function deleteAllEmojiReactions(channelId: string, messageId: string, emoji: SendEmoji): Promise<void> {
    return OpWrappers.discord_delete_all_reactions_for_emoji(channelId, messageId, emoji);
}

// Interactions
export async function getInteractionFollowupMessage(token: string, messageId: string): Promise<Message> {
    return new Message(await OpWrappers.getInteractionFollowupMessage(token, messageId));
}

export async function createInteractionFollowupMessage(token: string, resp: string | InteractionCreateMessageFields): Promise<Message> {
    let flags: InteractionMessageFlags = {}
    if (arguments.length === 3) {
        // legacy support, remove at some point in the future
        flags = arguments[2];
    } else {
        if (typeof resp === "object") {
            if (resp.flags) {
                flags = resp.flags
            }
        }
    }

    if (typeof resp === "string") {
        return new Message(await OpWrappers.createInteractionFollowupMessage({
            interactionToken: token,
            fields: { content: resp },
            flags: flags || {},
        }))
    } else {
        return new Message(await OpWrappers.createInteractionFollowupMessage({
            interactionToken: token,
            fields: toOpMessageFields(resp),
            flags: flags || {},
        }))
    }
}

export async function editInteractionFollowupMessage(token: string, messageId: string, fields: InteractionCreateMessageFields): Promise<void> {
    return await OpWrappers.editInteractionFollowupMessage(messageId, {
        interactionToken: token,
        fields: toOpMessageFields(fields),
        flags: fields.flags ?? {},
    })
}

export async function deleteInteractionFollowupMessage(token: string, id: string): Promise<void> {
    return OpWrappers.deleteInteractionFollowupMessage(token, id);
}

export async function getInteractionOriginalResponse(token: string): Promise<Message> {
    return new Message(await OpWrappers.getInteractionOriginal(token));
}

export async function editInteractionOriginalResponse(token: string, fields: InteractionCreateMessageFields): Promise<Message> {
    return new Message(await OpWrappers.editInteractionOriginal({
        interactionToken: token,
        fields: toOpMessageFields(fields),
        flags: fields.flags ?? {},
    }))
}

export async function deleteInteractionOriginalResponse(token: string): Promise<void> {
    return OpWrappers.deleteInteractionOriginal(token);
}

export interface ICreateStandaloneThread {
    /**
     * The channel the thread is in
     */
    channelId: string;

    /**
     * 1-100 characters long name for the thread
     */
    name: string;

    kind: "NewsThread" | "PublicThread" | "PrivateThread";

    /**
     * The thread will stop showing in the channel list after auto_archive_duration minutes of inactivity, can be set to: 60, 1440, 4320, 10080
     */
    autoArchiveDurationMinutes?: AutoArchiveMinutes;

    /**
     * Whether non-moderators can add other non-moderators to a thread; only available when creating a private thread
     */
    invitable?: boolean;
}

export async function createStandaloneThread(create: ICreateStandaloneThread): Promise<Thread> {
    return threadChannelFromInternal(await OpWrappers.callAsyncOp({
        kind: "discord_start_thread_without_message",
        arg: create,
    }))
}

export interface ICreateThreadFromMessage {
    /**
     * The channel the thread is in
     */
    channelId: string;

    /**
     * Message to start the thread on
     */
    messageId: string;

    /**
     * The thread will stop showing in the channel list after auto_archive_duration minutes of inactivity, can be set to: 60, 1440, 4320, 10080
     */
    autoArchiveDurationMinutes?: AutoArchiveMinutes;

    /**
     * 1-100 characters long name for the thread
     */
    name: string;
}

export async function createThreadFromMessage(create: ICreateThreadFromMessage): Promise<Thread> {
    return threadChannelFromInternal(await OpWrappers.callAsyncOp({
        kind: "discord_start_thread_from_message",
        arg: create,
    }))
}

export interface ICreateForumThread {
    /**
     * The channel the thread is in, has to be of type forum
     */
    channelId: string;

    /**
     * 1-100 characters long name for the thread
     */
    name: string;

    /**
     * Tags to assign to the thread
     */
    tagIds?: string[];

    /**
     * The thread will stop showing in the channel list after auto_archive_duration minutes of inactivity, can be set to: 60, 1440, 4320, 10080
     */
    autoArchiveDurationMinutes?: AutoArchiveMinutes;


    /**
     * The starting message to be created in the thread
     */
    message: CreateMessageFields;
}

export async function createForumThread(create: ICreateForumThread): Promise<{ thread: Thread; message: Message; }> {
    const opArg = {
        ...create,
        message: toOpMessageFields(create.message),
    }

    const thread = await OpWrappers.callAsyncOp({
        kind: "discord_start_forum_thread",
        arg: opArg,
    })

    return {
        thread: threadChannelFromInternal(thread.channel),
        message: new Message(thread.message)
    }
}

export interface IEditThread {
    /**
     * The id of the thread to edit
     */
    channelId: string;

    tagIds?: string[];

    /**
     * The thread will stop showing in the channel list after auto_archive_duration minutes of inactivity, can be set to: 60, 1440, 4320, 10080
     */
    autoArchiveDurationMinutes?: AutoArchiveMinutes;


    /**
     * Whether non-moderators can add other non-moderators to a thread; only available on private threads
     */
    invitable?: boolean;

    /**
     * 1-100 long name for the thread
     */
    name?: string;

    rateLimitPerUser?: number;

    /**
     * Whether the thread is locked; when a thread is locked, only users with MANAGE_THREADS can unarchive it
     */
    locked?: boolean,

    archived?: boolean,
}

export async function editThread(fields: IEditThread): Promise<Thread> {
    return threadChannelFromInternal(await OpWrappers.callAsyncOp({
        kind: "discord_edit_thread",
        arg: fields,
    }))
}

export async function addThreadMember(channelId: string, userId: string) {
    await OpWrappers.callAsyncOp({
        kind: "discord_add_thread_member",
        arg: [channelId, userId]
    })
}

export async function removeThreadMember(channelId: string, userId: string) {
    await OpWrappers.callAsyncOp({
        kind: "discord_remove_thread_member",
        arg: [channelId, userId]
    })
}



// discord_list_thread_members
export interface IListThreadMembers {
    /**
     * The thread id
     */
    channelId: string;

    /**
     * Get thread members after this user ID, used for pagination.
     */
    afterUserId?: string;

    /**
     * Max number of thread members to return (1-100). Defaults to 100.
     */
    limit?: number;

    /**
     * Whether to include a guild member object for each thread member
     */
    withMember?: boolean;
}

export async function getThreadMembers(options: IListThreadMembers): Promise<ThreadMember[]> {
    let resp = await OpWrappers.callAsyncOp({
        kind: "discord_list_thread_members",
        arg: options,
    })

    return resp.map(v => new ThreadMember(v))
}

export interface IThreadsListing {
    /**
     * Whether there are potentially additional threads that could be returned on a subsequent call
     */
    hasMore?: boolean;

    /**
     * This only includes the bot member, if you you want to list all the members in a thread, see {@link getThreadMembers}
     */
    members: ThreadMember[];

    threads: Thread[];
}

export async function getActiveThreads(): Promise<Omit<IThreadsListing, "hasMore">> {
    let resp = await OpWrappers.callAsyncOp({
        kind: "discord_list_active_threads",
        arg: null,
    })

    return {
        // hasMore: resp.hasMore,
        members: resp.members.map(v => new ThreadMember(v)),
        threads: resp.threads.map(v => threadChannelFromInternal(v))
    }
}

export interface IListThreads {
    /**
     * Channel to return threads from
     */
    channelId: string;

    /**
     * Unix timestamp in milliseconds to returns threads before, used for pagination. 
     */
    before?: number;
}

export async function getPublicArchivedThreads(options: IListThreads): Promise<IThreadsListing> {
    let resp = await OpWrappers.callAsyncOp({
        kind: "discord_list_public_archived_threads",
        arg: options,
    })

    return {
        hasMore: resp.hasMore,
        members: resp.members.map(v => new ThreadMember(v)),
        threads: resp.threads.map(v => threadChannelFromInternal(v))
    }
}

export async function getPrivateArchivedThreads(options: IListThreads): Promise<IThreadsListing> {
    let resp = await OpWrappers.callAsyncOp({
        kind: "discord_list_private_archived_threads",
        arg: options,
    })

    return {
        hasMore: resp.hasMore,
        members: resp.members.map(v => new ThreadMember(v)),
        threads: resp.threads.map(v => threadChannelFromInternal(v))
    }
}
