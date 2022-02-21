import type { AutoArchiveDuration } from "./AutoArchiveDuration";
import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface PublicThread {
  default_auto_archive_duration: AutoArchiveDuration | null;
  guild_id: string;
  id: string;
  kind: "PublicThread";
  last_message_id: string | null;
  member: ThreadMember | null;
  member_count: number;
  message_count: number;
  name: string;
  owner_id: string | null;
  parent_id: string | null;
  rate_limit_per_user: number | null;
  thread_metadata: ThreadMetadata;
}
