import type { IThreadMember } from "./ThreadMember";
import type { PermissionOverwrite } from "../discord/PermissionOverwrite";
import type { ThreadMetadata } from "../discord/ThreadMetadata";

export interface IPrivateThread {
  defaultAutoArchiveDurationMinutes: number | null;
  id: string;
  invitable: boolean | null;
  kind: "PrivateThread";
  member: IThreadMember | null;
  memberCount: number;
  messageCount: number;
  name: string;
  ownerId: string | null;
  parentId: string | null;
  permissionOverwrites: Array<PermissionOverwrite>;
  rateLimitPerUser: number | null;
  threadMetadata: ThreadMetadata;
}
