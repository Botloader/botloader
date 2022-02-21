import type { AutoArchiveDuration } from "./AutoArchiveDuration";
import type { ThreadMember } from "./ThreadMember";
import type { ThreadMetadata } from "./ThreadMetadata";

export interface NewsThread {
  default_auto_archive_duration: AutoArchiveDuration | null;
  id: string;
  kind: "NewsThread";
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
