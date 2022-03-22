import type { PermissionOverwrite } from "../discord/PermissionOverwrite";
import type { VideoQualityMode } from "../discord/VideoQualityMode";

export interface IVoiceChannel {
  bitrate: number;
  id: string;
  kind: "Voice" | "StageVoice";
  name: string;
  parentId: string | null;
  permissionOverwrites: Array<PermissionOverwrite>;
  position: bigint;
  rtcRegion: string | null;
  userLimit: number | null;
  videoQualityMode: VideoQualityMode | null;
}
