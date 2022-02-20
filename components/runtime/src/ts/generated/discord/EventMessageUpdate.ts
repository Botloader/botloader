import type { Attachment } from "./Attachment";
import type { Embed } from "./Embed";
import type { Mention } from "./Mention";
import type { MessageType } from "./MessageType";
import type { User } from "./User";

export interface EventMessageUpdate {
  attachments?: Array<Attachment>;
  author?: User;
  channelId: string;
  content: string | null;
  editedTimestamp?: number;
  embeds?: Array<Embed>;
  guildId?: string;
  id: string;
  kind?: MessageType;
  mentionEveryone?: boolean;
  mentionRoles?: Array<string>;
  mentions?: Array<Mention>;
  pinned?: boolean;
  timestamp?: number;
  tts?: boolean;
}
