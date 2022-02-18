import type { CommandInteractionDataMap } from "./CommandInteractionDataMaps";
import type { CommandInteractionOption } from "./CommandInteractionOption";
import type { Member } from "../discord/Member";

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
