import type { Member } from "./Member";
import type { ReactionType } from "./ReactionType";

export interface EventMessageReactionAdd {
  channelId: string;
  messageId: string;
  emoji: ReactionType;
  member: Member;
  userId: string;
}
