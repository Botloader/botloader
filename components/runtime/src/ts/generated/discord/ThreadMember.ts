import type { Member } from "./Member";

export interface ThreadMember {
  id: string | null;
  join_timestamp: number;
  member: Member | null;
  user_id: string | null;
}
