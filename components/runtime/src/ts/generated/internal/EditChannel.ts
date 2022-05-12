import type { PermissionOverwrite } from "../discord/PermissionOverwrite";
import type { VideoQualityMode } from "../discord/VideoQualityMode";

export interface IEditChannel {
  bitrate?: number;
  name?: string;
  nsfw?: boolean;
  parentId?: string | null;
  permissionOverwrites?: Array<PermissionOverwrite>;
  position?: number;
  rateLimitPerUser?: number;
  topic?: string;
  userLimit?: number;
  videoQualityMode?: VideoQualityMode;
}
