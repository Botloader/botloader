import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface CategoryChannel {
  id: string;
  kind: "Category";
  name: string;
  permissionOverwrites: Array<PermissionOverwrite>;
  position: bigint;
}
