import type { IPermissionOverwrite } from "../discord/IPermissionOverwrite";
import type { VideoQualityMode } from "../discord/VideoQualityMode";

export interface IEditChannel {
  bitrate?: number;
  name?: string;
  nsfw?: boolean;
  parentId?: string | null;
  permissionOverwrites?: Array<IPermissionOverwrite>;
  position?: number;
  rateLimitPerUser?: number;
  topic?: string;
  userLimit?: number;
  videoQualityMode?: VideoQualityMode;
}
