import type { CommandOption } from "./CommandOption";
import type { CommandType } from "./CommandType";

export interface Command {
  name: string;
  description: string;
  options: Array<CommandOption>;
  group?: string;
  subGroup?: string;
  kind: CommandType;
}
