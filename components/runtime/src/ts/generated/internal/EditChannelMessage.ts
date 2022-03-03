import type { OpCreateMessageFields } from "./CreateMessageFields";

export interface OpEditChannelMessage {
  channelId: string;
  messageId: string;
  fields: OpCreateMessageFields;
}
