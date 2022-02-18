import type { CategoryChannel } from "./CategoryChannel";
import type { NewsThread } from "./NewsThread";
import type { PrivateThread } from "./PrivateThread";
import type { PublicThread } from "./PublicThread";
import type { TextChannel } from "./TextChannel";
import type { VoiceChannel } from "./VoiceChannel";

export type GuildChannel =
  | CategoryChannel
  | NewsThread
  | PrivateThread
  | PublicThread
  | TextChannel
  | VoiceChannel
  | VoiceChannel;
