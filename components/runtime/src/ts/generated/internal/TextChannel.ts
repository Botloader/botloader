import type { IPermissionOverwrite } from "../discord/IPermissionOverwrite";

export interface ITextChannel {
  id: string;
  kind: "Text" | "News" | "Store" | "Forum" | "GuildDirectory";
  lastPinTimestamp: number | null;
  name: string;
  nsfw: boolean;
  parentId: string | null;
  permissionOverwrites: Array<IPermissionOverwrite>;
  position: bigint;
  rateLimitPerUser: number | null;
  topic: string | null;
}
