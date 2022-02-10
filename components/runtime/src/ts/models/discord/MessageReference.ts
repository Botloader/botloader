export interface MessageReference {
  channelId: string | null;
  guildId: string | null;
  messageId: string | null;
  failIfNotExists: boolean | null;
}
