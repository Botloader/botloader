import type { PermissionOverwrite } from "./PermissionOverwrite";

export interface TextChannel {
  guild_id: string;
  id: string;
  kind: "GuildText" | "GuildNews" | "GuildStore";
  last_message_id: string | null;
  last_pin_timestamp: number | null;
  name: string;
  nsfw: boolean;
  parent_id: string | null;
  permission_overwrites: Array<PermissionOverwrite>;
  position: bigint;
  rate_limit_per_user: number | null;
  topic: string | null;
}
