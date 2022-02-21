import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface CategoryChannel {
  guild_id: string;
  id: string;
  kind: "Category";
  name: string;
  permission_overwrites: Array<PermissionOverwrite>;
  position: bigint;
}
