import * as Internal from "./generated/internal/index";
import * as Discord from './generated/discord/index';

// This file contains op wrappers
// They are used internally and you shoudl NEVER use them in your own scripts
// if you do so your script WILL break at some point in the future when this gets changed
// (and this changes a lot)

export namespace OpWrappers {

    export namespace utils {
        export function encodeText(s: string): Uint8Array {
            return Deno.core.opSync(
                "op_bl_encode_text",
                s
            );
        }

        export function decodeText(u: Uint8Array): string {
            return Deno.core.opSync(
                "op_bl_decode_text",
                u
            );
        }
    }

    export namespace http {
        export function createRequestStream(): number {
            return Deno.core.opSync("op_bl_http_client_stream")
        }

        export function requestSend(args: Internal.ClientHttpRequest): Promise<Internal.ClientHttpResponse> {
            return Deno.core.opAsync("op_bl_http_request_send", args)
        }
    }

    export namespace tasks {
        export function scheduleTask(data: Internal.CreateScheduledTask): Promise<Internal.ScheduledTask> {
            return Deno.core.opAsync("op_bl_schedule_task", data)
        }

        export function delTask(taskId: number): Promise<boolean> {
            return Deno.core.opAsync("op_bl_del_task", taskId)
        }

        export function delTaskByKey(name: string, key: string): Promise<boolean> {
            return Deno.core.opAsync("op_bl_del_task_by_key", name, key)
        }

        export function delAllTasks(name: string): Promise<number> {
            return Deno.core.opAsync("op_bl_del_all_tasks", name)
        }

        export function getTask(taskId: number): Promise<Internal.ScheduledTask | null> {
            return Deno.core.opAsync("op_bl_get_task", taskId)
        }

        export function getTaskByKey(name: string, key: string): Promise<Internal.ScheduledTask | null> {
            return Deno.core.opAsync("op_bl_get_task_by_key", name, key)
        }

        export function getAllTasks(name: string | undefined, after_id: number): Promise<Internal.ScheduledTask[]> {
            return Deno.core.opAsync("op_bl_get_all_tasks", name, after_id)
        }
    }

    export function scriptStarted(meta: Internal.ScriptMeta) {
        Deno.core.opSync(
            "op_botloader_script_start",
            meta
        );
    }

    export function consoleLog(args: Internal.ConsoleLogMessage) {
        Deno.core.opSync(
            "op_botloader_log",
            args
        );
    }

    export async function getGuild(): Promise<Discord.Guild> {
        return Deno.core.opAsync("op_discord_get_guild");
    }

    export function getCurrentUser(): Internal.IUser {
        return Deno.core.opSync("op_get_current_bot_user");
    }

    export function getCurrentGuildId(): string {
        return Deno.core.opSync("op_get_current_guild_id");
    }

    // Messages
    export async function getMessage(channelId: string, messageId: string): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_get_message",
            channelId,
            messageId,
        );
    }

    export async function getMessages(args: Internal.OpGetMessages): Promise<Internal.IMessage[]> {
        return await Deno.core.opAsync(
            "op_discord_get_messages",
            args
        );
    }

    export async function createChannelMessage(args: Internal.OpCreateChannelMessage): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_create_message",
            args
        );
    }

    export async function crosspostChannelMessage(channelId: string, messageId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_crosspost_message",
            channelId,
            messageId
        );
    }

    export async function editChannelMessage(args: Internal.OpEditChannelMessage): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_edit_message",
            args
        );
    }

    export async function deleteChannelMessage(args: Internal.OpDeleteMessage): Promise<void> {
        await Deno.core.opAsync(
            "op_discord_delete_message",
            args
        );
    }
    export async function deleteChannelMessagesBulk(args: Internal.OpDeleteMessagesBulk): Promise<void> {
        await Deno.core.opAsync(
            "op_discord_bulk_delete_messages",
            args
        );
    }

    // Interactions
    export async function interactionCallback(args: Internal.InteractionCallback): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_interaction_callback",
            args
        );
    }

    export async function getInteractionFollowupMessage(token: string, messageId: string): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_interaction_get_followup_message",
            token,
            messageId
        );
    }


    export async function createInteractionFollowupMessage(args: Internal.OpCreateFollowUpMessage): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_interaction_followup_message",
            args
        );
    }

    export async function editInteractionFollowupMessage(messageId: string, args: Internal.OpCreateFollowUpMessage): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_interaction_edit_followup_message",
            messageId,
            args,
        );
    }

    export async function deleteInteractionFollowupMessage(token: string, messageId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_interaction_delete_followup_message",
            token,
            messageId,
        );
    }

    export async function getInteractionOriginal(token: string): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_interaction_get_original_response",
            token
        );
    }

    export async function editInteractionOriginal(args: Internal.OpCreateFollowUpMessage): Promise<Internal.IMessage> {
        return await Deno.core.opAsync(
            "op_discord_interaction_edit_original_response",
            args
        );
    }

    export async function deleteInteractionOriginal(token: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_interaction_delete_original",
            token,
        );
    }

    // Roles
    export async function getRole(roleId: string): Promise<Discord.Role> {
        return await Deno.core.opAsync(
            "op_discord_get_role",
            roleId
        );
    }

    export async function getRoles(): Promise<Discord.Role[]> {
        return await Deno.core.opAsync(
            "op_discord_get_roles",
        );
    }

    // Channels
    export async function getChannels(): Promise<Internal.InternalGuildChannel[]> {
        return await Deno.core.opAsync(
            "op_discord_get_channels",
        );
    }

    export async function getChannel(channelId: string): Promise<Internal.InternalGuildChannel> {
        return await Deno.core.opAsync(
            "op_discord_get_channel",
            channelId,
        );
    }

    export async function editChannel(channelId: string, fields: Internal.IEditChannel): Promise<Internal.InternalGuildChannel> {
        return await Deno.core.opAsync(
            "op_discord_edit_channel",
            channelId,
            fields,
        );
    }

    export async function createChannel(fields: Internal.ICreateChannel): Promise<Internal.InternalGuildChannel> {
        return await Deno.core.opAsync(
            "op_discord_create_channel",
            fields,
        );
    }

    export async function deleteChannel(channelId: string): Promise<Internal.InternalGuildChannel> {
        return await Deno.core.opAsync(
            "op_discord_delete_channel",
            channelId,
            channelId,
        );
    }

    // Pins
    export async function op_discord_get_channel_pins(channelId: string): Promise<Internal.IMessage[]> {
        return await Deno.core.opAsync(
            "op_discord_get_channel_pins",
            channelId,
        );
    }

    export async function op_discord_create_pin(channelId: string, messageId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_create_pin",
            channelId,
            messageId,
        );
    }
    export async function op_discord_delete_pin(channelId: string, messageId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_delete_pin",
            channelId,
            messageId,
        );
    }

    // Members
    export async function getMembers(ids: string[]): Promise<(Internal.IMember | null)[]> {
        return await Deno.core.opAsync(
            "op_discord_get_members",
            ids,
        );
    }

    export async function updateMember(userId: string, fields: Internal.UpdateGuildMemberFields): Promise<Internal.IMember> {
        return await Deno.core.opAsync(
            "op_discord_update_member",
            userId,
            fields
        );
    }

    export async function addMemberRole(userId: string, roleId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_add_member_role",
            userId,
            roleId,
        );
    }

    export async function removeMemberRole(userId: string, roleId: string): Promise<void> {
        return await Deno.core.opAsync(
            "op_discord_remove_member_role",
            userId,
            roleId,
        );
    }
    export async function removeMember(userId: string, extras: Discord.AuditLogExtras): Promise<void> {
        return await Deno.core.opAsync("op_discord_remove_member", userId, extras);
    }

    export async function getMemberPermissions(userId: string, roles: string[] | null, channelId: string | null): Promise<[string, string | null]> {
        return await Deno.core.opAsync("op_discord_get_member_permissions", userId, [roles, channelId]);
    }

    // Storage
    export async function bucketStorageSet(opts: Internal.OpStorageBucketSetValue): Promise<Internal.OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set", opts);
    }

    export async function bucketStorageSetIf(opts: Internal.OpStorageBucketSetIf): Promise<Internal.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set_if", opts);
    }

    export async function bucketStorageGet(opts: Internal.OpStorageBucketEntryId): Promise<Internal.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_get", opts);
    }

    export async function bucketStorageDel(opts: Internal.OpStorageBucketEntryId): Promise<Internal.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_del", opts);
    }

    export async function bucketStorageDelMany(bucketName: string, keyPattern: string): Promise<number> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_del_many", bucketName, keyPattern);
    }

    export async function bucketStorageList(opts: Internal.OpStorageBucketList): Promise<Internal.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_list", opts);
    }

    export async function bucketStorageCount(bucketName: string, keyPattern: string): Promise<number> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_count", bucketName, keyPattern);
    }

    export async function bucketStorageIncr(opts: Internal.OpStorageBucketIncr): Promise<Internal.OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_incr", opts);
    }

    export async function bucketStorageSortedList(opts: Internal.OpStorageBucketSortedList): Promise<Internal.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_sorted_list", opts);
    }

    // Bans
    export async function createBan(userId: string, extras: Internal.CreateBanFields): Promise<void> {
        return await Deno.core.opAsync("op_discord_create_ban", userId, extras);
    }

    export async function getBan(userId: string): Promise<Internal.IBan> {
        return Deno.core.opAsync("op_discord_get_ban", userId);
    }

    export async function getBans(): Promise<Internal.IBan[]> {
        return Deno.core.opAsync("op_discord_get_bans");
    }

    export async function removeBan(userId: string, extras: Discord.AuditLogExtras): Promise<void> {
        return await Deno.core.opAsync("op_discord_delete_ban", userId, extras);
    }

    // Reactions
    export async function discord_create_reaction(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return Deno.core.opAsync("op_discord_create_reaction", [channelId, messageId], emoji)
    }
    export async function discord_delete_own_reaction(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return Deno.core.opAsync("op_discord_delete_own_reaction", [channelId, messageId], emoji)
    }
    export async function discord_delete_user_reaction(channelId: string, messageId: string, userId: string, emoji: Discord.SendEmoji): Promise<void> {
        return Deno.core.opAsync("op_discord_delete_user_reaction", [channelId, messageId, userId], emoji)
    }
    export async function discord_get_reactions(channelId: string, messageId: string, fields: Internal.GetReactionsFields): Promise<Internal.IUser[]> {
        return Deno.core.opAsync("op_discord_get_reactions", [channelId, messageId], fields)
    }
    export async function discord_delete_all_reactions(channelId: string, messageId: string): Promise<void> {
        return Deno.core.opAsync("op_discord_delete_all_reactions", [channelId, messageId])
    }
    export async function discord_delete_all_reactions_for_emoji(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return Deno.core.opAsync("op_discord_delete_all_reactions_for_emoji", [channelId, messageId], emoji)
    }
}