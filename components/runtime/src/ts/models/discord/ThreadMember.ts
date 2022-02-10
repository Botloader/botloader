import type { Member } from "./Member";

export interface ThreadMember {
  flags: number;
  id: string | null;
  join_timestamp: number;
  member: Member | null;
  user_id: string | null;
}
