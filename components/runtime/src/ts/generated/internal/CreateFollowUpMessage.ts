import type { OpCreateMessageFields } from "./CreateMessageFields";

export interface OpCreateFollowUpMessage {
  interactionToken: string;
  fields: OpCreateMessageFields;
}
