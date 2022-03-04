import type { ComponentType } from "../discord/ComponentType";
import type { Member } from "../discord/Member";
import type { Message } from "../discord/Message";

export interface MessageComponentInteraction {
  channelId: string;
  guildLocale: string | null;
  id: string;
  locale: string;
  member: Member;
  message: Message;
  token: string;
  customId: string;
  componentType: ComponentType;
  values: Array<string>;
}
