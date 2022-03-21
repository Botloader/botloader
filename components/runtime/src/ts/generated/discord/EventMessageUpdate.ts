import type { Attachment } from "./Attachment";
import type { Embed } from "./Embed";
import type { IUser } from "../internal/IUser";
import type { MessageType } from "./MessageType";
import type { UserMention } from "./UserMention";

export interface EventMessageUpdate {
  attachments?: Array<Attachment>;
  author?: IUser;
  channelId: string;
  content: string | null;
  editedTimestamp?: number;
  embeds?: Array<Embed>;
  guildId?: string;
  id: string;
  kind?: MessageType;
  mentionEveryone?: boolean;
  mentionRoles?: Array<string>;
  mentions?: Array<UserMention>;
  pinned?: boolean;
  timestamp?: number;
  tts?: boolean;
}
