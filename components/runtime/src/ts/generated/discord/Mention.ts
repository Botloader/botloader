import type { PartialMember } from "./PartialMember";

export interface Mention {
  avatar: string | null;
  bot: boolean;
  discriminator: number;
  id: string;
  member: PartialMember | null;
  username: string;
  publicFlags: number;
}
