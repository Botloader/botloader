import type { IUserFlags } from "./IUserFlags";
import type { PremiumType } from "./PremiumType";

export interface IUser {
  avatar: string | null;
  bot: boolean;
  discriminator: string;
  id: string;
  locale: string | null;
  username: string;
  premiumType: PremiumType | null;
  publicFlags: IUserFlags | null;
  system: boolean | null;
}
