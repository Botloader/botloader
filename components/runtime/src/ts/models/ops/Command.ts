import type { CommandOption } from "./CommandOption";

export interface Command {
  name: string;
  description: string;
  options: Array<CommandOption>;
  group?: string;
  subGroup?: string;
}
