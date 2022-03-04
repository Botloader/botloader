export interface MessageFlags {
  crossposted?: boolean;
  isCrosspost?: boolean;
  suppressEmbeds?: boolean;
  sourceMessageDeleted?: boolean;
  urgent?: boolean;
  hasThread?: boolean;
  ephemeral?: boolean;
  loading?: boolean;
  failedToMentionSomeRolesInThread?: boolean;
}
