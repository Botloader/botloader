import type { PartialMember } from "./PartialMember";
import type { UserFlags } from "./UserFlags";

export interface UserMention {
  avatar: string | null;
  bot: boolean;
  discriminator: number;
  id: string;
  member: PartialMember | null;
  username: string;
  publicFlags: UserFlags;
}
