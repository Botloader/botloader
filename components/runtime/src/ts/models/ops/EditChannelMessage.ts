import type { OpEditMessageFields } from "./EditMessageFields";

export interface OpEditChannelMessage {
  channelId: string;
  messageId: string;
  fields: OpEditMessageFields;
}
