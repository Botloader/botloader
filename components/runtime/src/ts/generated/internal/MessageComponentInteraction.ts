import type { ComponentType } from "../discord/ComponentType";
import type { IMember } from "./Member";
import type { IMessage } from "./IMessage";

export interface MessageComponentInteraction {
  channelId: string;
  guildLocale: string | null;
  id: string;
  locale: string;
  member: IMember;
  message: IMessage;
  token: string;
  customId: string;
  componentType: ComponentType;
  values: Array<string>;
}
