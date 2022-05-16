import type { ChannelType } from "./ChannelType";

export interface IEventThreadDelete {
  guildId: string;
  id: string;
  kind: ChannelType;
  parentId: string;
}
