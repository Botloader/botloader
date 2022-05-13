import type { IPermissionOverwrite } from "../discord/IPermissionOverwrite";

export interface ICategoryChannel {
  id: string;
  kind: "Category";
  name: string;
  permissionOverwrites: Array<IPermissionOverwrite>;
  position: bigint;
}
