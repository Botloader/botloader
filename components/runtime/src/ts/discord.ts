export * from './generated/discord/index';
import { Guild, GuildChannel, Member, Message, Role, Embed, Component } from './generated/discord/index';
import * as Internal from './generated/internal/index';
import { OpWrappers } from './op_wrappers';

let a: GuildChannel | null = null;


// Guild functions
export function getGuild(): Promise<Guild> {
    return OpWrappers.getGuild()
}
function editGuild() { }

// Message functions
export function getMessage(channelId: string, messageId: string): Promise<Message> {
    return OpWrappers.getMessage({
        channelId,
        messageId,
    })
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

export function getMessages(channelId: string, options?: GetMessagesOptions): Promise<Message[]> {
    return OpWrappers.getMessages({
        channelId,
        after: options?.after,
        before: options?.before,
        limit: options?.limit,
    })
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

export function createMessage(channelId: string, fields: CreateMessageFields): Promise<Message> {
    return OpWrappers.createChannelMessage({
        channelId,
        fields: toOpMessageFields(fields),
    });
}
export function editMessage(channelId: string, messageId: string, fields: CreateMessageFields): Promise<Message> {
    return OpWrappers.editChannelMessage({
        channelId,
        messageId,
        fields: toOpMessageFields(fields),
    });
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
export function getChannel(channelId: string): Promise<GuildChannel> {
    return OpWrappers.getChannel(channelId);
}
export function getChannels(): Promise<GuildChannel[]> {
    return OpWrappers.getChannels();
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
    return (await OpWrappers.getMembers([id]))[0] || undefined;
}

export async function getMembers(ids: string[]): Promise<(Member | null)[]> {
    return await OpWrappers.getMembers(ids);
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
    return await OpWrappers.updateMember(userId, fields);
}

export async function addMemberRole(userId: string, roleId: string): Promise<void> {
    return await OpWrappers.addMemberRole(userId, roleId);
}

export async function removeMemberRole(userId: string, roleId: string): Promise<void> {
    return await OpWrappers.removeMemberRole(userId, roleId);
}

