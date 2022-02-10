import type { PermissionOverwriteType } from "./PermissionOverwriteType";

export interface PermissionOverwrite {
  allow: string;
  deny: string;
  kind: PermissionOverwriteType;
  id: string;
}
