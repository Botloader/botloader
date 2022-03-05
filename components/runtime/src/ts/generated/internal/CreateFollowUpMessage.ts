import type { MessageFlags } from "../discord/MessageFlags";
import type { OpCreateMessageFields } from "./CreateMessageFields";

export interface OpCreateFollowUpMessage {
  interactionToken: string;
  fields: OpCreateMessageFields;
  flags: MessageFlags | null;
}
