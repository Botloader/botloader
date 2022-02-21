import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface PublicThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  kind: "PublicThread";
  member: ThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
