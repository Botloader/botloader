// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CustomEmoji } from "./CustomEmoji";
import type { DiscordWebhook } from "./DiscordWebhook";
import type { Guild } from "../discord/Guild";
import type { IForumThreadResponse } from "./IForumThreadResponse";
import type { IMessage } from "./IMessage";
import type { IThreadMember } from "./ThreadMember";
import type { IThreadsListing } from "./IThreadsListing";
import type { InternalGuildChannel } from "./GuildChannel";
import type { Role } from "../discord/Role";

export interface EasyOpsReturnTypesASync {
  discord_get_guild: Guild;
  discord_get_message: IMessage;
  discord_get_messages: Array<IMessage>;
  discord_create_message: IMessage;
  discord_edit_message: IMessage;
  discord_crosspost_message: null;
  discord_delete_message: null;
  discord_bulk_delete_messages: null;
  discord_start_thread_from_message: InternalGuildChannel;
  discord_start_thread_without_message: InternalGuildChannel;
  discord_start_forum_thread: IForumThreadResponse;
  discord_add_thread_member: null;
  discord_remove_thread_member: null;
  discord_list_thread_members: Array<IThreadMember>;
  discord_list_active_threads: IThreadsListing;
  discord_list_public_archived_threads: IThreadsListing;
  discord_list_private_archived_threads: IThreadsListing;
  discord_edit_thread: InternalGuildChannel;
  discord_bulk_edit_channels: null;
  discord_create_role: Role;
  discord_update_role: Role;
  discord_update_role_positions: Array<Role>;
  discord_delete_role: null;
  discord_get_emojis: Array<CustomEmoji>;
  discord_create_emoji: CustomEmoji;
  discord_edit_emoji: CustomEmoji;
  discord_delete_emoji: null;
  discord_webhook_get: DiscordWebhook;
  discord_webhook_get_guild: Array<DiscordWebhook>;
  discord_webhook_create: DiscordWebhook;
  discord_webhook_edit: DiscordWebhook;
  discord_webhook_edit_with_token: DiscordWebhook;
  discord_webhook_delete: null;
  discord_webhook_execute: IMessage;
  discord_webhook_message_get: IMessage;
  discord_webhook_message_delete: null;
  discord_webhook_message_edit: IMessage;
}
