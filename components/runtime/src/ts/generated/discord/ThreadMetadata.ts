export interface ThreadMetadata {
  archived: boolean;
  autoArchiveDurationMinutes: number;
  archiveTimestamp: number;
  invitable: boolean | null;
  locked: boolean;
}
