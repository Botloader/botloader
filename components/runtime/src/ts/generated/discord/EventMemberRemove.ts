import type { User } from "./User";

export interface EventMemberRemove {
  guildId: string;
  user: User;
}
