import type { ReactionType } from "./ReactionType";

export interface MessageReaction {
  count: number;
  emoji: ReactionType;
  me: boolean;
}
