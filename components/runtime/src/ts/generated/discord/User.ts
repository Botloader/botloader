import type { PremiumType } from "./PremiumType";
import type { UserFlags } from "./UserFlags";

export interface User {
  avatar: string | null;
  bot: boolean;
  discriminator: number;
  id: string;
  locale: string | null;
  username: string;
  premiumType: PremiumType | null;
  publicFlags: UserFlags | null;
  system: boolean | null;
}
