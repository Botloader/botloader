import type { User } from "../discord/User";

export interface MemberRemove {
  guildId: string;
  user: User;
}
