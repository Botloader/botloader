import { Internal } from "./generated";
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
        return Deno.core.opAsync("discord_get_guild");
    }

    export async function getMessage(args: Internal.OpGetMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_get_message",
            args
        );
    }

    export async function getMessages(args: Internal.OpGetMessages): Promise<Discord.Message[]> {
        return await Deno.core.opAsync(
            "discord_get_messages",
            args
        );
    }

    export async function createChannelMessage(args: Internal.OpCreateChannelMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_create_message",
            args
        );
    }

    export async function editChannelMessage(args: Internal.OpEditChannelMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_edit_message",
            args
        );
    }

    export async function deleteChannelMessage(args: Internal.OpDeleteMessage): Promise<void> {
        await Deno.core.opAsync(
            "discord_delete_message",
            args
        );
    }
    export async function deleteChannelMessagesBulk(args: Internal.OpDeleteMessagesBulk): Promise<void> {
        await Deno.core.opAsync(
            "discord_bulk_delete_messages",
            args
        );
    }

    export async function createInteractionFollowup(args: Internal.OpCreateFollowUpMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_create_followup_message",
            args
        );
    }

    export async function interactionCallback(args: Internal.InteractionCallback): Promise<void> {
        return await Deno.core.opAsync(
            "discord_interaction_callback",
            args
        );
    }

    export async function deleteInteractionFollowup(token: string, messageId: string): Promise<void> {
        return await Deno.core.opAsync(
            "discord_interaction_delete_followup",
            token,
            messageId,
        );
    }

    export async function deleteInteractionOriginal(token: string): Promise<void> {
        return await Deno.core.opAsync(
            "discord_interaction_delete_original",
            token,
        );
    }

    export async function getRole(roleId: string): Promise<Discord.Role> {
        return await Deno.core.opAsync(
            "discord_get_role",
            roleId
        );
    }

    export async function getRoles(): Promise<Discord.Role[]> {
        return await Deno.core.opAsync(
            "discord_get_roles",
        );
    }

    export async function getChannels(): Promise<Discord.GuildChannel[]> {
        return await Deno.core.opAsync(
            "discord_get_channels",
        );
    }

    export async function getChannel(channelId: string): Promise<Discord.GuildChannel> {
        return await Deno.core.opAsync(
            "discord_get_channel",
            channelId,
        );
    }

    export async function getMembers(ids: string[]): Promise<(Discord.Member | null)[]> {
        return await Deno.core.opAsync(
            "discord_get_members",
            ids,
        );
    }

    export async function updateMember(userId: string, fields: Internal.UpdateGuildMemberFields): Promise<Discord.Member> {
        return await Deno.core.opAsync(
            "discord_update_member",
            userId,
            fields
        );
    }

    export async function addMemberRole(userId: string, roleId: string): Promise<void> {
        return await Deno.core.opAsync(
            "discord_add_member_role",
            userId,
            roleId,
        );
    }

    export async function removeMemberRole(userId: string, roleId: string): Promise<void> {
        return await Deno.core.opAsync(
            "discord_remove_member_role",
            userId,
            roleId,
        );
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

    export async function bucketStorageList(opts: Internal.OpStorageBucketList): Promise<Internal.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_list", opts);
    }

    export async function bucketStorageIncr(opts: Internal.OpStorageBucketIncr): Promise<Internal.OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_incr", opts);
    }

    export async function bucketStorageSortedList(opts: Internal.OpStorageBucketSortedList): Promise<Internal.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_sorted_list", opts);
    }

}