import type { PermissionOverwrite } from "./PermissionOverwrite";
import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface PrivateThread {
  default_auto_archive_duration_minutes: number | null;
  id: string;
  invitable: boolean | null;
  kind: "PrivateThread";
  member: ThreadMember | null;
  member_count: number;
  message_count: number;
  name: string;
  owner_id: string | null;
  parent_id: string | null;
  permission_overwrites: Array<PermissionOverwrite>;
  rate_limit_per_user: number | null;
  thread_metadata: ThreadMetadata;
}
