import type { IUser } from "./IUser";

export interface IEventMemberRemove {
  guildId: string;
  user: IUser;
}
