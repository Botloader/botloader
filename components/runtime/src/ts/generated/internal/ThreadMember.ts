import type { IMember } from "./Member";

export interface IThreadMember {
  id: string | null;
  joinTimestamp: number;
  member: IMember | null;
  userId: string | null;
}
