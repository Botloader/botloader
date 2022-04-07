import type { IMember } from "./Member";
import type { IMessage } from "./IMessage";
import type { IModalInteractionDataComponent } from "./IModalInteractionDataComponent";

export interface IModalInteraction {
  channelId: string;
  guildLocale: string | null;
  id: string;
  locale: string;
  member: IMember;
  message: IMessage | null;
  token: string;
  customId: string;
  values: Array<IModalInteractionDataComponent>;
}
