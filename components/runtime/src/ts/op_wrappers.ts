import * as Internal from "./generated/internal/index";
import * as Discord from './generated/discord/index';
import { VoiceState } from "./discord/events";

// This file contains op wrappers
// They are used internally and you should NEVER use them in your own scripts
// if you do so your script WILL break at some point in the future when this gets changed
// (and this changes a lot)

const {
    op_bl_http_request_send,
    op_bl_schedule_task,
    op_bl_del_task,
    op_bl_del_task_by_key,
    op_bl_del_all_tasks,
    op_bl_get_task,
    op_bl_get_task_by_key,
    op_bl_get_all_tasks,
    // op_discord_get_guild,
    op_discord_get_invites,
    op_discord_get_invite,
    op_discord_delete_invite,
    // op_discord_get_message,
    // op_discord_get_messages,
    // op_discord_create_message,
    // op_discord_crosspost_message,
    // op_discord_edit_message,
    // op_discord_delete_message,
    // op_discord_bulk_delete_messages,
    op_discord_interaction_callback,
    op_discord_interaction_get_followup_message,
    op_discord_interaction_followup_message,
    op_discord_interaction_edit_followup_message,
    op_discord_interaction_delete_followup_message,
    op_discord_interaction_get_original_response,
    op_discord_interaction_edit_original_response,
    op_discord_interaction_delete_original,
    op_discord_get_role,
    op_discord_get_roles,
    op_discord_get_channels,
    op_discord_get_channel,
    op_discord_edit_channel,
    op_discord_create_channel,
    op_discord_delete_channel,
    op_discord_update_channel_permission,
    op_discord_delete_channel_permission,
    op_discord_get_channel_pins: _op_discord_get_channel_pins,
    op_discord_create_pin: _op_discord_create_pin,
    op_discord_delete_pin: _op_discord_delete_pin,
    op_discord_get_voice_states,
    op_discord_get_members,
    op_discord_update_member,
    op_discord_add_member_role,
    op_discord_remove_member_role,
    op_discord_remove_member,
    op_discord_get_member_permissions,
    op_botloader_bucket_storage_set,
    op_botloader_bucket_storage_get,
    op_botloader_bucket_storage_del,
    op_botloader_bucket_storage_del_many,
    op_botloader_bucket_storage_list,
    op_botloader_bucket_storage_count,
    op_botloader_bucket_storage_incr,
    op_botloader_bucket_storage_sorted_list,
    op_botloader_bucket_storage_set_if,
    op_discord_create_ban,
    op_discord_get_ban,
    op_discord_get_bans,
    op_discord_delete_ban,
    op_discord_create_reaction,
    op_discord_delete_own_reaction,
    op_discord_delete_user_reaction,
    op_discord_get_reactions,
    op_discord_delete_all_reactions,
    op_discord_delete_all_reactions_for_emoji,
    op_discord_get_channel_invites,
    op_discord_create_channel_invite,
    op_easyops_async,
} = Deno.core.ensureFastOps()

export namespace OpWrappers {

    export namespace utils {
        export function encodeText(s: string): Uint8Array {
            return Deno.core.ops.op_bl_encode_text(
                s
            );
        }

        export function decodeText(u: Uint8Array): string {
            return Deno.core.ops.op_bl_decode_text(
                u
            );
        }
    }

    export namespace http {
        export function createRequestStream(): number {
            return Deno.core.ops.op_bl_http_client_stream()
        }

        export function requestSend(args: Internal.ClientHttpRequest): Promise<Internal.ClientHttpResponse> {
            return op_bl_http_request_send(args)
        }
    }

    export namespace tasks {
        export function scheduleTask(data: Internal.CreateScheduledTask): Promise<Internal.ScheduledTask> {
            return op_bl_schedule_task(data)
        }

        export function delTask(taskId: number): Promise<boolean> {
            return op_bl_del_task(taskId)
        }

        export function delTaskByKey(pluginId: string | null, name: string, key: string): Promise<boolean> {
            return op_bl_del_task_by_key(pluginId, name, key)
        }

        export function delAllTasks(pluginId: string | null, name: string): Promise<number> {
            return op_bl_del_all_tasks(pluginId, name)
        }

        export function getTask(taskId: number): Promise<Internal.ScheduledTask | null> {
            return op_bl_get_task(taskId)
        }

        export function getTaskByKey(pluginId: string | null, name: string, key: string): Promise<Internal.ScheduledTask | null> {
            return op_bl_get_task_by_key(pluginId, name, key)
        }

        export function getAllTasks(filter: Internal.GetGuildTasksFilter, after_id: number): Promise<Internal.ScheduledTask[]> {
            return op_bl_get_all_tasks(filter, after_id)
        }
    }

    export function scriptStarted(meta: Internal.ScriptMeta) {
        Deno.core.ops.op_botloader_script_start(
            meta
        );
    }

    export function getSettings() {
        return Deno.core.ops.op_get_settings()
    }

    export function consoleLog(args: Internal.ConsoleLogMessage) {
        Deno.core.ops.op_botloader_log(
            args
        );
    }

    export async function callAsyncOp<T extends Internal.EasyOpsASync>(call: T): Promise<Internal.EasyOpsReturnTypesASync[T["kind"]]> {
        return await op_easyops_async(call)
    }

    // export async function getGuild(): Promise<Discord.Guild> {
    //     return op_discord_get_guild();
    // }

    export async function getInvites(): Promise<Internal.IInvite[]> {
        return await op_discord_get_invites();
    }

    export async function getInvite(code: string, with_counts: boolean, with_expiration: boolean): Promise<Internal.IInvite> {
        return await op_discord_get_invite(code, with_counts, with_expiration);
    }

    export async function deleteInvite(code: string): Promise<void> {
        return await op_discord_delete_invite(code);
    }


    export function getCurrentUser(): Internal.IUser {
        return Deno.core.ops.op_get_current_bot_user();
    }

    export function getCurrentGuildId(): string {
        return Deno.core.ops.op_get_current_guild_id();
    }

    // Messages
    // export async function getMessage(channelId: string, messageId: string): Promise<Internal.IMessage> {
    //     return await op_discord_get_message(
    //         channelId,
    //         messageId,
    //     );
    // }

    // export async function getMessages(args: Internal.OpGetMessages): Promise<Internal.IMessage[]> {
    //     return await op_discord_get_messages(
    //         args
    //     );
    // }

    // export async function createChannelMessage(args: Internal.OpCreateChannelMessage): Promise<Internal.IMessage> {
    //     return await op_discord_create_message(
    //         args
    //     );
    // }

    // export async function crosspostChannelMessage(channelId: string, messageId: string): Promise<void> {
    //     return await op_discord_crosspost_message(
    //         channelId,
    //         messageId
    //     );
    // }

    // export async function editChannelMessage(args: Internal.OpEditChannelMessage): Promise<Internal.IMessage> {
    //     return await op_discord_edit_message(
    //         args
    //     );
    // }

    // export async function deleteChannelMessage(args: Internal.OpDeleteMessage): Promise<void> {
    //     await op_discord_delete_message(
    //         args
    //     );
    // }
    // export async function deleteChannelMessagesBulk(args: Internal.OpDeleteMessagesBulk): Promise<void> {
    //     await op_discord_bulk_delete_messages(
    //         args
    //     );
    // }

    // Interactions
    export async function interactionCallback(args: Internal.InteractionCallback): Promise<void> {
        return await op_discord_interaction_callback(
            args
        );
    }

    export async function getInteractionFollowupMessage(token: string, messageId: string): Promise<Internal.IMessage> {
        return await op_discord_interaction_get_followup_message(
            token,
            messageId
        );
    }


    export async function createInteractionFollowupMessage(args: Internal.OpCreateFollowUpMessage): Promise<Internal.IMessage> {
        return await op_discord_interaction_followup_message(
            args
        );
    }

    export async function editInteractionFollowupMessage(messageId: string, args: Internal.OpCreateFollowUpMessage): Promise<void> {
        return await op_discord_interaction_edit_followup_message(
            messageId,
            args,
        );
    }

    export async function deleteInteractionFollowupMessage(token: string, messageId: string): Promise<void> {
        return await op_discord_interaction_delete_followup_message(
            token,
            messageId,
        );
    }

    export async function getInteractionOriginal(token: string): Promise<Internal.IMessage> {
        return await op_discord_interaction_get_original_response(
            token
        );
    }

    export async function editInteractionOriginal(args: Internal.OpCreateFollowUpMessage): Promise<Internal.IMessage> {
        return await op_discord_interaction_edit_original_response(
            args
        );
    }

    export async function deleteInteractionOriginal(token: string): Promise<void> {
        return await op_discord_interaction_delete_original(
            token,
        );
    }

    // Roles
    export async function getRole(roleId: string): Promise<Discord.Role> {
        return await op_discord_get_role(
            roleId
        );
    }

    export async function getRoles(): Promise<Discord.Role[]> {
        return await op_discord_get_roles();
    }

    // Channels
    export async function getChannels(): Promise<Internal.InternalGuildChannel[]> {
        return await op_discord_get_channels();
    }

    export async function getChannel(channelId: string): Promise<Internal.InternalGuildChannel> {
        return await op_discord_get_channel(
            channelId,
        );
    }

    export async function editChannel(channelId: string, fields: Internal.IEditChannel): Promise<Internal.InternalGuildChannel> {
        return await op_discord_edit_channel(
            channelId,
            fields,
        );
    }

    export async function createChannel(fields: Internal.ICreateChannel): Promise<Internal.InternalGuildChannel> {
        return await op_discord_create_channel(
            fields,
        );
    }

    export async function deleteChannel(channelId: string): Promise<Internal.InternalGuildChannel> {
        return await op_discord_delete_channel(
            channelId,
        );
    }

    export async function updateChannelPermission(channelId: string, overwrite: Discord.IPermissionOverwrite): Promise<void> {
        return await op_discord_update_channel_permission(
            channelId,
            overwrite,
        );
    }

    export async function deleteChannelPermission(channelId: string, kind: Discord.PermissionOverwriteType, id: string): Promise<void> {
        return await op_discord_delete_channel_permission(
            channelId,
            [kind, id],
        );
    }

    export async function getChannelInvites(channelId: string): Promise<Internal.IInvite[]> {
        return await op_discord_get_channel_invites(
            channelId,
        );
    }

    export async function createChannelInvite(channelId: string, fields: Internal.ICreateInviteFields): Promise<Internal.IInvite> {
        return await op_discord_create_channel_invite(
            channelId,
            fields,
        );
    }

    // Pins
    export async function op_discord_get_channel_pins(channelId: string): Promise<Internal.IMessage[]> {
        return await _op_discord_get_channel_pins(
            channelId,
        );
    }

    export async function op_discord_create_pin(channelId: string, messageId: string): Promise<void> {
        return await _op_discord_create_pin(
            channelId,
            messageId,
        );
    }
    export async function op_discord_delete_pin(channelId: string, messageId: string): Promise<void> {
        return await _op_discord_delete_pin(
            channelId,
            messageId,
        );
    }

    export async function getVoiceStates(): Promise<VoiceState[]> {
        return await op_discord_get_voice_states();
    }

    // Members
    export async function getMembers(ids: string[]): Promise<(Internal.IMember | null)[]> {
        return await op_discord_get_members(
            ids,
        );
    }

    export async function updateMember(userId: string, fields: Internal.UpdateGuildMemberFields): Promise<Internal.IMember> {
        return await op_discord_update_member(
            userId,
            fields
        );
    }

    export async function addMemberRole(userId: string, roleId: string): Promise<void> {
        return await op_discord_add_member_role(
            userId,
            roleId,
        );
    }

    export async function removeMemberRole(userId: string, roleId: string): Promise<void> {
        return await op_discord_remove_member_role(
            userId,
            roleId,
        );
    }
    export async function removeMember(userId: string, extras: Discord.AuditLogExtras): Promise<void> {
        return await op_discord_remove_member(userId, extras);
    }

    export async function getMemberPermissions(userId: string, roles: string[] | null, channelId: string | null): Promise<[string, string | null]> {
        return await op_discord_get_member_permissions(userId, [roles, channelId]);
    }

    // Storage
    export async function bucketStorageSet(opts: Internal.OpStorageBucketSetValue): Promise<Internal.OpStorageBucketEntry> {
        return await op_botloader_bucket_storage_set(opts);
    }

    export async function bucketStorageSetIf(opts: Internal.OpStorageBucketSetIf): Promise<Internal.OpStorageBucketEntry | null> {
        return await op_botloader_bucket_storage_set_if({
            ...opts,
        });
    }

    export async function bucketStorageGet(opts: Internal.OpStorageBucketEntryId): Promise<Internal.OpStorageBucketEntry | null> {
        return await op_botloader_bucket_storage_get(opts);
    }

    export async function bucketStorageDel(opts: Internal.OpStorageBucketEntryId): Promise<Internal.OpStorageBucketEntry | null> {
        return await op_botloader_bucket_storage_del(opts);
    }

    export async function bucketStorageDelMany(pluginId: string | null, bucketName: string, keyPattern: string): Promise<number> {
        return await op_botloader_bucket_storage_del_many(pluginId, bucketName, keyPattern);
    }

    export async function bucketStorageList(opts: Internal.OpStorageBucketList): Promise<Internal.OpStorageBucketEntry[]> {
        return await op_botloader_bucket_storage_list(opts);
    }

    export async function bucketStorageCount(pluginId: string | null, bucketName: string, keyPattern: string): Promise<number> {
        return await op_botloader_bucket_storage_count(pluginId, bucketName, keyPattern);
    }

    export async function bucketStorageIncr(opts: Internal.OpStorageBucketIncr): Promise<Internal.OpStorageBucketEntry> {
        return await op_botloader_bucket_storage_incr(opts);
    }

    export async function bucketStorageSortedList(opts: Internal.OpStorageBucketSortedList): Promise<Internal.OpStorageBucketEntry[]> {
        return await op_botloader_bucket_storage_sorted_list(opts);
    }

    // Bans
    export async function createBan(userId: string, extras: Internal.CreateBanFields): Promise<void> {
        return await op_discord_create_ban(userId, extras);
    }

    export async function getBan(userId: string): Promise<Internal.IBan> {
        return op_discord_get_ban(userId);
    }

    export async function getBans(): Promise<Internal.IBan[]> {
        return op_discord_get_bans();
    }

    export async function removeBan(userId: string, extras: Discord.AuditLogExtras): Promise<void> {
        return await op_discord_delete_ban(userId, extras);
    }

    // Reactions
    export async function discord_create_reaction(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return op_discord_create_reaction([channelId, messageId], emoji)
    }
    export async function discord_delete_own_reaction(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return op_discord_delete_own_reaction([channelId, messageId], emoji)
    }
    export async function discord_delete_user_reaction(channelId: string, messageId: string, userId: string, emoji: Discord.SendEmoji): Promise<void> {
        return op_discord_delete_user_reaction([channelId, messageId, userId], emoji)
    }
    export async function discord_get_reactions(channelId: string, messageId: string, fields: Internal.GetReactionsFields): Promise<Internal.IUser[]> {
        return op_discord_get_reactions([channelId, messageId], fields)
    }
    export async function discord_delete_all_reactions(channelId: string, messageId: string): Promise<void> {
        return op_discord_delete_all_reactions([channelId, messageId])
    }
    export async function discord_delete_all_reactions_for_emoji(channelId: string, messageId: string, emoji: Discord.SendEmoji): Promise<void> {
        return op_discord_delete_all_reactions_for_emoji([channelId, messageId], emoji)
    }
}

