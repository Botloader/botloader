export interface PartialMember {
  deaf: boolean;
  joinedAt: number;
  mute: boolean;
  nick: string | null;
  premiumSince: number | null;
  roles: Array<string>;
}
