import type { ChannelType } from "./ChannelType";

export interface ChannelMention {
  guildId: string;
  id: string;
  kind: ChannelType;
  name: string;
}
