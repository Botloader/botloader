import type { ComponentType } from "../discord/ComponentType";
import type { IMessage } from "../discord/IMessage";
import type { Member } from "../discord/Member";

export interface MessageComponentInteraction {
  channelId: string;
  guildLocale: string | null;
  id: string;
  locale: string;
  member: Member;
  message: IMessage;
  token: string;
  customId: string;
  componentType: ComponentType;
  values: Array<string>;
}
