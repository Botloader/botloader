import type { PermissionOverwrite } from "./PermissionOverwrite";
import type { VideoQualityMode } from "./VideoQualityMode";

export interface VoiceChannel {
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
