import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface TextChannel {
  id: string;
  kind: "Text" | "News" | "Store";
  lastPinTimestamp: number | null;
  name: string;
  nsfw: boolean;
  parentId: string | null;
  permissionOverwrites: Array<PermissionOverwrite>;
  position: bigint;
  rateLimitPerUser: number | null;
  topic: string | null;
}
