export interface ThreadMetadata {
  archived: boolean;
  auto_archive_duration_minutes: number;
  archive_timestamp: number;
  invitable: boolean | null;
  locked: boolean;
}
