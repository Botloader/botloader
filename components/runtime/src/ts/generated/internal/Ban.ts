import type { IUser } from "./IUser";

export interface IBan {
  reason: string | null;
  user: IUser;
}
