import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface CategoryChannel {
  id: string;
  kind: "Category";
  name: string;
  permission_overwrites: Array<PermissionOverwrite>;
  position: bigint;
}
