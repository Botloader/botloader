import type { ReactionType } from "./ReactionType";

export interface EventMessageReactionRemoveAllEmoji {
  channelId: string;
  messageId: string;
  emoji: ReactionType;
}
