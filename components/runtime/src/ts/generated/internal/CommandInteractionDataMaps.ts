import type { IMessage } from "../discord/IMessage";
import type { InteractionPartialChannel } from "./InteractionChannel";
import type { InteractionPartialMember } from "./InteractionPartialMember";
import type { Role } from "../discord/Role";
import type { User } from "../discord/User";

export interface CommandInteractionDataMap {
  channels: Record<string, InteractionPartialChannel>;
  members: Record<string, InteractionPartialMember>;
  messages: Record<string, IMessage>;
  roles: Record<string, Role>;
  users: Record<string, User>;
}
