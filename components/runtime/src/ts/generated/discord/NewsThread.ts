import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface NewsThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  kind: "NewsThread";
  member: ThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
