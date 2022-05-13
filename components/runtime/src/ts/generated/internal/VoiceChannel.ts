import type { IPermissionOverwrite } from "../discord/IPermissionOverwrite";
import type { VideoQualityMode } from "../discord/VideoQualityMode";

export interface IVoiceChannel {
  bitrate: number;
  id: string;
  kind: "Voice" | "StageVoice";
  name: string;
  parentId: string | null;
  permissionOverwrites: Array<IPermissionOverwrite>;
  position: bigint;
  rtcRegion: string | null;
  userLimit: number | null;
  videoQualityMode: VideoQualityMode | null;
}
