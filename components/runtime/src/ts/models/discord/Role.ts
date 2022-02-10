import type { RoleTags } from "./RoleTags";

export interface Role {
  color: number;
  hoist: boolean;
  icon: string | null;
  id: string;
  managed: boolean;
  mentionable: boolean;
  name: string;
  permissions: string;
  position: bigint;
  tags: RoleTags | null;
  unicodeEmoji: string | null;
}
