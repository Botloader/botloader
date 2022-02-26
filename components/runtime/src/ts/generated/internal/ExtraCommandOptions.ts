import type { ChannelType } from "../discord/ChannelType";

export interface ExtraCommandOptions {
  minValue?: number;
  maxValue?: number;
  channelTypes?: Array<ChannelType>;
}
