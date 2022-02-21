import type { PermissionOverwriteType } from "./PermissionOverwriteType";

export interface PermissionOverwrite {
  allowRaw: string;
  denyRaw: string;
  kind: PermissionOverwriteType;
  id: string;
}
