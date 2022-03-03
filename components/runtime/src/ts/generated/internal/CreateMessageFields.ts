import type { AllowedMentions } from "./AllowedMentions";
import type { Component } from "../discord/Component";
import type { Embed } from "../discord/Embed";

export interface OpCreateMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
  components?: Array<Component>;
}
