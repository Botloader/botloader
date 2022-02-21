import type { PermissionOverwrite } from "./PermissionOverwrite";
import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface PrivateThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  invitable: boolean | null;
  kind: "PrivateThread";
  member: ThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  permissionOverwrites: Array<PermissionOverwrite>;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
