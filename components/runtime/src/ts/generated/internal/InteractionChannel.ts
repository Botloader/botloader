import type { ChannelType } from "../discord/ChannelType";
import type { ThreadMetadata } from "../discord/ThreadMetadata";

export interface InteractionPartialChannel {
  id: string;
  kind: ChannelType;
  name: string;
  parentId?: string;
  permissionsRaw: string;
  threadMetadata?: ThreadMetadata;
}
