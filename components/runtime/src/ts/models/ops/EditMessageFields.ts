import type { Embed } from "../discord/Embed";
import type { AllowedMentions } from "./AllowedMentions";

export interface OpEditMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
}
