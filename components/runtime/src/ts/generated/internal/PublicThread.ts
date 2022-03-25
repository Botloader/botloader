import type { ISelfThreadMember } from "./ISelfThreadMember";
import type { ThreadMetadata } from "../discord/ThreadMetadata";

export interface IPublicThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  kind: "PublicThread";
  member: ISelfThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
