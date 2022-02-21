import type { PremiumType } from "./PremiumType";
import type { UserFlags } from "./UserFlags";

export interface User {
  avatar: string | null;
  bot: boolean;
  discriminator: number;
  email: string | null;
  id: string;
  locale: string | null;
  mfaEnabled: boolean | null;
  username: string;
  premiumType: PremiumType | null;
  publicFlags: UserFlags | null;
  system: boolean | null;
  verified: boolean | null;
}
