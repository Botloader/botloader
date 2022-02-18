import type { AllowedMentions } from "./AllowedMentions";
import type { Embed } from "../discord/Embed";

export interface OpCreateMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
}
