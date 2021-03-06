import type { ButtonStyle } from "./ButtonStyle";
import type { ReactionType } from "./ReactionType";

export interface IButton {
  customId?: string;
  style: ButtonStyle;
  disabled?: boolean;
  url?: string;
  label?: string;
  emoji?: ReactionType;
}
