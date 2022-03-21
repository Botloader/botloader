import type { IUserFlags } from "../internal/IUserFlags";
import type { PartialMember } from "./PartialMember";

export interface UserMention {
  avatar: string | null;
  bot: boolean;
  discriminator: number;
  id: string;
  member: PartialMember | null;
  username: string;
  publicFlags: IUserFlags;
}
