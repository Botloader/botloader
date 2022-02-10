import type { PrivateThread } from "./PrivateThread";
import type { TextChannel } from "./TextChannel";
import type { PublicThread } from "./PublicThread";
import type { VoiceChannel } from "./VoiceChannel";
import type { NewsThread } from "./NewsThread";
import type { CategoryChannel } from "./CategoryChannel";

export type GuildChannel =
  | CategoryChannel
  | NewsThread
  | PrivateThread
  | PublicThread
  | TextChannel
  | VoiceChannel
  | VoiceChannel;
