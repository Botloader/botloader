import type { AutoArchiveDuration } from "./AutoArchiveDuration";

export interface ThreadMetadata {
  archived: boolean;
  auto_archive_duration: AutoArchiveDuration;
  archive_timestamp: number;
  invitable: boolean | null;
  locked: boolean;
}
