import type { AllowedMentions } from "./AllowedMentions";
import type { Embed } from "../discord/Embed";
import type { IComponent } from "../discord/IComponent";

export interface OpCreateMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
  components?: Array<IComponent>;
}
