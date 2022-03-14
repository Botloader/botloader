export * from './generated/discord/index';
import { Discord } from './docs_index';
import { Guild, GuildChannel, Member, Message, Role, Embed, Component, ComponentType, AuditLogExtras, SendEmoji, User } from './generated/discord/index';
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

export interface InteractionCreateMessageFields extends CreateMessageFields {
    flags?: InteractionMessageFlags,
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
    return (await OpWrappers.getMembers([id]))[0] ?? undefined;
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

export async function removeMember(userId: string, extras?: AuditLogExtras) {
    return OpWrappers.removeMember(userId, extras ?? {});
}


// Guild bans
export interface CreateBanExtras extends AuditLogExtras {
    deleteMessageDays: number
}

export async function createBan(userId: string, extras?: CreateBanExtras): Promise<void> {
    return OpWrappers.createBan(userId, extras ?? {});
}

export async function getBan(userID: string) {
    return OpWrappers.getBan(userID);
}

export async function getBans() {
    return OpWrappers.getBans();
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
    return OpWrappers.discord_get_reactions(channelId, messageId, {
        ...extra,
        emoji: emoji,
    });
}
export async function deleteAllReactions(channelId: string, messageId: string): Promise<void> {
    return OpWrappers.discord_delete_all_reactions(channelId, messageId);
}
export async function deleteAllEmojiReactions(channelId: string, messageId: string, emoji: SendEmoji): Promise<void> {
    return OpWrappers.discord_delete_all_reactions_for_emoji(channelId, messageId, emoji);
}

// Interactions
export async function getInteractionFollowupMessage(token: string, messageId: string) {
    return OpWrappers.getInteractionFollowupMessage(token, messageId);
}

export async function createInteractionFollowupMessage(token: string, resp: string | InteractionCreateMessageFields) {
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
        return await OpWrappers.createInteractionFollowupMessage({
            interactionToken: token,
            fields: { content: resp },
            flags: flags || {},
        })
    } else {
        return await OpWrappers.createInteractionFollowupMessage({
            interactionToken: token,
            fields: toOpMessageFields(resp),
            flags: flags || {},
        })
    }
}

export async function editInteractionFollowupMessage(token: string, messageId: string, fields: InteractionCreateMessageFields) {
    return await OpWrappers.editInteractionFollowupMessage(messageId, {
        interactionToken: token,
        fields: toOpMessageFields(fields),
        flags: fields.flags ?? {},
    })
}

export async function deleteInteractionFollowupMessage(token: string, id: string) {
    return OpWrappers.deleteInteractionFollowupMessage(token, id);
}

export async function getInteractionOriginalResponse(token: string) {
    return OpWrappers.getInteractionOriginal(token);
}

export async function editInteractionOriginalResponse(token: string, fields: InteractionCreateMessageFields) {
    return await OpWrappers.editInteractionOriginal({
        interactionToken: token,
        fields: toOpMessageFields(fields),
        flags: fields.flags ?? {},
    })
}

export async function deleteInteractionOriginalResponse(token: string) {
    return OpWrappers.deleteInteractionOriginal(token);
}

/**
 * Base interaction class, this class should be considered UNSTABLE and may change a lot in the future.
 */
export class Interaction {
    interactionId: string;
    token: string;

    /**
     * The user that started the interaction
     */
    member: Member;

    protected _hasSentCallback = false;

    get hasSentCallback() {
        return this._hasSentCallback;
    }

    constructor(id: string, token: string, member: Member) {
        this.interactionId = id;
        this.member = member;
        this.token = token;
    }

    protected setCallbackSent() {
        if (this.hasSentCallback) {
            throw new Error("tried sending a callback when one has already been sent, only one callback per interaction can be sent.")
        } else {
            this._hasSentCallback = true;
        }
    }

    /**
     * @deprecated use {@link ackWithMessage} 
     */
    async sendCallbackWithMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.ackWithMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * @deprecated use {@link ackWithDeferredMessage} 
     */
    async sendCallbackWithDeferredMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        this.ackWithDeferredMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * Acknowledge this interaction and send a message in response to this interaction
     */
    async ackWithMessage(fields: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "ChannelMessageWithSource",
                fields: toOpMessageFields(fields),
                flags: fields.flags || {},
            }
        })
    }

    /**
     * Acknowledge this interaction and display a "thinking" state to the user for you to then send a followUp 
     * message later.
     * 
     * You have to ack interactions within 3 seconds but if you are doing things that can take longer than that you can
     * use this function first to tell discord that you are processing the interaction then send the message itself later.
     */
    async ackWithDeferredMessage(fields?: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "DeferredChannelMessageWithSource",
                fields: toOpMessageFields(fields ?? {}),
                flags: fields?.flags ?? {},
            }
        })
    }

    /**
     * @deprecated use {@link sendFollowup} instead
     */
    async sendResponse(resp: string | CreateMessageFields) {
        return this.sendFollowup(resp);
    }

    async getOriginalResponse() {
        return getInteractionOriginalResponse(this.token);
    }

    async editOriginalResponse(fields: InteractionCreateMessageFields) {
        return editInteractionOriginalResponse(this.token, fields)
    }

    async deleteOriginalResponse() {
        return deleteInteractionOriginalResponse(this.token);
    }

    async getFollowup(messageId: string) {
        return getInteractionFollowupMessage(this.token, messageId);
    }

    async sendFollowup(resp: string | InteractionCreateMessageFields) {
        return createInteractionFollowupMessage(this.token, resp);
    }

    async createFollowup(resp: string | InteractionCreateMessageFields) {
        return createInteractionFollowupMessage(this.token, resp);
    }

    async editFollowup(messageId: string, fields: InteractionCreateMessageFields) {
        return editInteractionFollowupMessage(this.token, messageId, fields);
    }

    async deleteFollowup(id: string) {
        return deleteInteractionFollowupMessage(this.token, id);
    }
}

export class ComponentInteraction extends Interaction {
    customIdRaw: string;
    componentType: ComponentType;
    channelId: string;

    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction.id, interaction.token, interaction.member);

        this.componentType = interaction.componentType;
        this.customIdRaw = interaction.customId;
        this.channelId = interaction.channelId;
    }

    /**
     * @deprecated use {@link ackWithUpdateMessage}
     */
    async sendCallbackUpdateMessage(fields: CreateMessageFields, flags?: InteractionMessageFlags) {
        return this.ackWithUpdateMessage({
            ...fields,
            flags: flags,
        })
    }

    /**
     * @deprecated use {@link ackWithDeferredUpdateMessage}
     */
    async sendCallbackDeferredUpdateMessage() {
        return this.ackWithDeferredUpdateMessage();
    }

    /**
     * Acknowledge this interaction and update the message the component was on
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithUpdateMessage(fields: InteractionCreateMessageFields) {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "UpdateMessage",
                fields: toOpMessageFields(fields),
                flags: fields.flags || {},
            }
        })
    }

    /**
     * Acknowledge this interaction and update the message the component was on at a later time (within 15 mins).
     * 
     * You have to ack interactions within 3 seconds but if you are doing things that can take longer than that you can
     * use this function first to tell discord that you are processing the interaction then update the message later.
     * 
     * Use updateOriginalResponse to update the message
     */
    async ackWithDeferredUpdateMessage() {
        this.setCallbackSent();

        return OpWrappers.interactionCallback({
            interactionId: this.interactionId,
            ineractionToken: this.token,
            data: {
                kind: "DeferredUpdateMessage",
            }
        })
    }
}

export class SelectMenuInteraction extends ComponentInteraction {
    values: string[];


    constructor(interaction: Internal.MessageComponentInteraction) {
        super(interaction);

        this.values = interaction.values;
    }
}

export interface InteractionMessageFlags {
    /**
     * Ephemeral messages can only be seen by the author of the interaction
     */
    ephemeral?: boolean,
}

/**
 * Creates a 'customId' for you to use in message component's 'customId fields
 * 
 * This is needed as otherwise the interaction will not be handled by botloader.
 * 
 * DO NOT try to emulate this function yourself, we may change to different techniques entirely in the future and if you try to 
 * emulate this function them by implmenting it yourself you WILL break stuff.
 * 
 * Note that name + json(data) has to be less than 80 characters
 * 
 * @param name Name of the component, this is used when creating listeners using {@link Script.onInteractionButton} and {@link Script.onInteractionSelectMenu}
 * @param data Arbitrary data that will be passed to the interaction handlers, can be used to track a small amount of state.
 * Note that name + json(data) has to be less than 80 characters
 * @returns The customId for use in the customId field
 */
export function encodeInteractionCustomId(name: string, data: any) {
    let res = name + ":";
    if (data !== undefined && data !== null) {
        res += JSON.stringify(data);
    }

    // The string iterator that is used here iterates over characters,
    // not mere code units
    let length = [...res].length;
    if (res.length >= 80) {
        throw new Error("name + JSON.stringify(data) exceeds 80 characters")
    }

    return "0:" + res
}