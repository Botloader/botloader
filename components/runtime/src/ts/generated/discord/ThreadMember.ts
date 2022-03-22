import type { IMember } from "../internal/Member";

export interface ThreadMember {
  id: string | null;
  joinTimestamp: number;
  member: IMember | null;
  userId: string | null;
}
