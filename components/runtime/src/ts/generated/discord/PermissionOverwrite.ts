import type { PermissionOverwriteType } from "./PermissionOverwriteType";

export interface PermissionOverwrite {
  allow_raw: string;
  deny_raw: string;
  kind: PermissionOverwriteType;
  id: string;
}
