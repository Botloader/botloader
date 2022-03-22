import type { PermissionOverwrite } from "../discord/PermissionOverwrite";

export interface ITextChannel {
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
