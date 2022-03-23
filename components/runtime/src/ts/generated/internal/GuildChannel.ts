import type { ICategoryChannel } from "./CategoryChannel";
import type { INewsThread } from "./NewsThread";
import type { IPrivateThread } from "./PrivateThread";
import type { IPublicThread } from "./PublicThread";
import type { ITextChannel } from "./TextChannel";
import type { IVoiceChannel } from "./VoiceChannel";

export type InternalGuildChannel =
  | ICategoryChannel
  | INewsThread
  | IPrivateThread
  | IPublicThread
  | ITextChannel
  | IVoiceChannel
  | IVoiceChannel;
