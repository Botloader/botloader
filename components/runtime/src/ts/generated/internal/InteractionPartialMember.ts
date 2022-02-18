export interface InteractionPartialMember {
  joinedAt: number;
  nick: string | null;
  premiumSince?: number;
  roles: Array<string>;
}
