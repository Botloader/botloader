import { Guild, Role, Embed, Component, AuditLogExtras, SendEmoji } from '../generated/discord/index';
import * as Internal from '../generated/internal/index';
import { OpWrappers } from '../op_wrappers';
import { GuildChannel, guildChannelFromInternal } from './channel';
import { Ban, Member } from './member';
import { Message } from './message';
import { User } from './user';

// Guild functions
export function getGuild(): Promise<Guild> {
    return OpWrappers.getGuild()
}
function editGuild() { }

// Message functions
export async function getMessage(channelId: string, messageId: string): Promise<Message> {
    return new Message(await OpWrappers.getMessage({
        channelId,
        messageId,
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
    return (await OpWrappers.getMessages({
        channelId,
        after: options?.after,
        before: options?.before,
        limit: options?.limit,
    })).map(v => new Message(v));
}

export interface CreateMessageFields {
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

    components?: Component[],
}

export interface InteractionCreateMessageFields extends CreateMessageFields {
    flags?: InteractionMessageFlags,
}

export interface InteractionMessageFlags {
    /**
     * Ephemeral messages can only be seen by the author of the interaction
     */
    ephemeral?: boolean,
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

    return new Message(await OpWrappers.createChannelMessage({
        channelId,
        fields: toOpMessageFields(fields),
    }));
}
export async function editMessage(channelId: string, messageId: string, fields: CreateMessageFields): Promise<Message> {
    return new Message(await OpWrappers.editChannelMessage({
        channelId,
        messageId,
        fields: toOpMessageFields(fields),
    }));
}

export function deleteMessage(channelId: string, messageId: string): Promise<void> {
    return OpWrappers.deleteChannelMessage({
        channelId,
        messageId,
    })
}

export function bulkDeleteMessages(channelId: string, ...messageIds: string[]): Promise<void> {
    return OpWrappers.deleteChannelMessagesBulk({
        channelId,
        messageIds,
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

async function createChannel() { }
async function editChannel() { }
async function deleteChannel() { }

// Invite functions
async function getInvite() { }
async function getInvites() { }
async function createInvite() { }
async function deleteInvite() { }

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
}

export async function editMember(userId: string, fields: UpdateGuildMemberFields): Promise<Member> {
    return new Member(await OpWrappers.updateMember(userId, fields));
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
