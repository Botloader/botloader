import type { Embed } from "../discord/Embed";
import type { AllowedMentions } from "./AllowedMentions";

export interface OpCreateMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
}
