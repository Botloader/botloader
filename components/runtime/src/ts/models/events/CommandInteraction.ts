import type { CommandInteractionDataMap } from "./CommandInteractionDataMaps";
import type { Member } from "../discord/Member";
import type { CommandInteractionOption } from "./CommandInteractionOption";

export interface CommandInteraction {
  channelId: string;
  id: string;
  member: Member;
  token: string;
  name: string;
  parentName: string | null;
  parentParentName: string | null;
  options: Array<CommandInteractionOption>;
  dataMap: CommandInteractionDataMap;
}
