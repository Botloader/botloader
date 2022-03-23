import type { IMember } from "./Member";
import type { ReactionType } from "../discord/ReactionType";

export interface IEventMessageReactionAdd {
  channelId: string;
  messageId: string;
  emoji: ReactionType;
  member: IMember;
  userId: string;
}
