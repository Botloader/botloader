import type { OpCreateMessageFields } from "./CreateMessageFields";

export interface OpCreateChannelMessage {
  channelId: string;
  fields: OpCreateMessageFields;
}
