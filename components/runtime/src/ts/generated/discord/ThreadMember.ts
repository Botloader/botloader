import type { Member } from "./Member";

export interface ThreadMember {
  id: string | null;
  joinTimestamp: number;
  member: Member | null;
  userId: string | null;
}
