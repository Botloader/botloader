export interface UpdateGuildMemberFields {
  channelId?: string | null;
  deaf?: boolean;
  mute?: boolean;
  nick?: string | null;
  roles?: string[];
  communicationDisabledUntil?: number | null;
}
