import type { IUser } from "../internal/IUser";

export interface EventMemberRemove {
  guildId: string;
  user: IUser;
}
