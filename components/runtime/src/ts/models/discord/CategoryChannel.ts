import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface CategoryChannel {
  guild_id: string;
  id: string;
  kind: "GuildCategory";
  name: string;
  permission_overwrites: Array<PermissionOverwrite>;
  position: bigint;
}
