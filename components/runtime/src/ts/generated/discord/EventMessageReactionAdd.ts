import type { IMember } from "../internal/Member";
import type { ReactionType } from "./ReactionType";

export interface EventMessageReactionAdd {
  channelId: string;
  messageId: string;
  emoji: ReactionType;
  member: IMember;
  userId: string;
}
