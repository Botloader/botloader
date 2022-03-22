import type { PermissionOverwrite } from "../discord/PermissionOverwrite";

export interface ICategoryChannel {
  id: string;
  kind: "Category";
  name: string;
  permissionOverwrites: Array<PermissionOverwrite>;
  position: bigint;
}
