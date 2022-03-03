import type { ButtonStyle } from "./ButtonStyle";
import type { ReactionType } from "./ReactionType";

export interface Button {
  customId?: string;
  disabled: boolean;
  style: ButtonStyle;
  url?: string;
  label?: string;
  emoji?: ReactionType;
}
