import type { Attachment } from "../discord/Attachment";
import type { Embed } from "../discord/Embed";
import type { IUser } from "./IUser";
import type { IUserMention } from "./UserMention";
import type { MessageType } from "../discord/MessageType";

export interface IEventMessageUpdate {
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
  mentions?: Array<IUserMention>;
  pinned?: boolean;
  timestamp?: number;
  tts?: boolean;
}
