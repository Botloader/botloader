import type { IUser } from "../internal/IUser";

export interface Ban {
  reason: string | null;
  user: IUser;
}
