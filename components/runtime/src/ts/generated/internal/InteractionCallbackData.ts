import type { MessageFlags } from "../discord/MessageFlags";
import type { OpCreateMessageFields } from "./CreateMessageFields";

export interface InteractionCallbackData {
  fields: OpCreateMessageFields;
  flags: MessageFlags | null;
}
