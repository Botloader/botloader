import type { PremiumType } from "./PremiumType";

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
  publicFlags: number | null;
  system: boolean | null;
  verified: boolean | null;
}
