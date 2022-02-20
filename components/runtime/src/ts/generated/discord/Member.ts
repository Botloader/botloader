import type { User } from "./User";

export interface Member {
  deaf: boolean;
  joined_at: number;
  mute: boolean;
  nick: string | null;
  pending: boolean;
  premium_since: number | null;
  roles: Array<string>;
  user: User;
}
