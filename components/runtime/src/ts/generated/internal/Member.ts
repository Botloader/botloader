import type { IUser } from "./IUser";

export interface IMember {
  deaf: boolean;
  joinedAt: number;
  mute: boolean;
  nick: string | null;
  pending: boolean;
  premiumSince: number | null;
  roles: Array<string>;
  user: IUser;
}
