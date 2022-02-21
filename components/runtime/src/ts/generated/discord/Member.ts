import type { User } from "./User";

export interface Member {
  deaf: boolean;
  joinedAt: number;
  mute: boolean;
  nick: string | null;
  pending: boolean;
  premiumSince: number | null;
  roles: Array<string>;
  user: User;
}
