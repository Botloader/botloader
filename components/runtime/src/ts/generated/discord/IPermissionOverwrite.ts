import type { PermissionOverwriteType } from "./PermissionOverwriteType";

export interface IPermissionOverwrite {
  allowRaw: string;
  denyRaw: string;
  kind: PermissionOverwriteType;
  id: string;
}
