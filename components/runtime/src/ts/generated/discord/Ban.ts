import type { User } from "./User";

export interface Ban {
  reason: string | null;
  user: User;
}
