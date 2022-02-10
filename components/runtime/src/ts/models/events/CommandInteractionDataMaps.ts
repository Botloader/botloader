import type { InteractionPartialChannel } from "./InteractionChannel";
import type { Role } from "../discord/Role";
import type { User } from "../discord/User";
import type { InteractionPartialMember } from "./InteractionPartialMember";
import type { Message } from "../discord/Message";

export interface CommandInteractionDataMap {
  channels: Record<string, InteractionPartialChannel>;
  members: Record<string, InteractionPartialMember>;
  messages: Record<string, Message>;
  roles: Record<string, Role>;
  users: Record<string, User>;
}
