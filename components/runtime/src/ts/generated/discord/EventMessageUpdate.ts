import type { Attachment } from "./Attachment";
import type { Embed } from "./Embed";
import type { Mention } from "./Mention";
import type { MessageType } from "./MessageType";
import type { User } from "./User";

export interface EventMessageUpdate {
  attachments: Array<Attachment> | null;
  author: User | null;
  channelId: string;
  content: string | null;
  editedTimestamp: number | null;
  embeds: Array<Embed> | null;
  guildId: string | null;
  id: string;
  kind: MessageType | null;
  mentionEveryone: boolean | null;
  mentionRoles: Array<string> | null;
  mentions: Array<Mention> | null;
  pinned: boolean | null;
  timestamp: number | null;
  tts: boolean | null;
}
