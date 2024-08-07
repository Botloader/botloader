// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ComponentType } from "../discord/ComponentType";
import type { IMember } from "./Member";
import type { IMessage } from "./IMessage";
import type { InteractionDataMap } from "./InteractionDataMaps";

export interface MessageComponentInteraction {
  channelId: string;
  guildLocale: string | null;
  id: string;
  locale: string;
  member: IMember;
  message: IMessage;
  token: string;
  resolved: InteractionDataMap | null;
  customId: string;
  componentType: ComponentType;
  values: Array<string>;
}
