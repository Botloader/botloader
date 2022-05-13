import type { ChannelType } from "../discord/ChannelType";
import type { IPermissionOverwrite } from "../discord/IPermissionOverwrite";

export interface ICreateChannel {
  name: string;
  kind?: ChannelType;
  bitrate?: number;
  nsfw?: boolean;
  parentId?: string;
  permissionOverwrites?: Array<IPermissionOverwrite>;
  position?: number;
  rateLimitPerUser?: number;
  topic?: string;
  userLimit?: number;
}
