import type { Embed } from "./Embed";
import type { ChannelMention } from "./ChannelMention";
import type { MessageReference } from "./MessageReference";
import type { MessageActivity } from "./MessageActivity";
import type { MessageType } from "./MessageType";
import type { Mention } from "./Mention";
import type { MessageApplication } from "./MessageApplication";
import type { Attachment } from "./Attachment";
import type { User } from "./User";
import type { MessageReaction } from "./MessageReaction";
import type { PartialMember } from "./PartialMember";

export interface Message {
  activity: MessageActivity | null;
  application: MessageApplication | null;
  attachments: Array<Attachment>;
  author: User;
  channelId: string;
  content: string;
  editedTimestamp: number | null;
  embeds: Array<Embed>;
  flags: number | null;
  guildId: string | null;
  id: string;
  kind: MessageType;
  member: PartialMember | null;
  mentionChannels: Array<ChannelMention>;
  mentionEveryone: boolean;
  mentionRoles: Array<string>;
  mentions: Array<Mention>;
  pinned: boolean;
  reactions: Array<MessageReaction>;
  reference: MessageReference | null;
  referencedMessage: Message | null;
  timestamp: number;
  tts: boolean;
  webhookId: string | null;
}
