import type { ReactionType } from "./ReactionType";

export interface EventMessageReactionRemove {
  channelId: string;
  messageId: string;
  emoji: ReactionType;
  userId: string;
}
