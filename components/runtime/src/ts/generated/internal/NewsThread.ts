import type { IThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "../discord/ThreadMetadata";

export interface INewsThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  kind: "NewsThread";
  member: IThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
