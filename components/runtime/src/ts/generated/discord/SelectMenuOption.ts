import type { ReactionType } from "./ReactionType";

export interface SelectMenuOption {
  default: boolean;
  description?: string;
  emoji?: ReactionType;
  label: string;
  value: string;
}
