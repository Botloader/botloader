import type { ReactionType } from "./ReactionType";

export interface ISelectMenuOption {
  default: boolean;
  description?: string;
  emoji?: ReactionType;
  label: string;
  value: string;
}
