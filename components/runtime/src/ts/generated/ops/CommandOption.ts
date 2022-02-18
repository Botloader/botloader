import type { CommandOptionType } from "./CommandOptionType";

export interface CommandOption {
  name: string;
  description: string;
  kind: CommandOptionType;
  required: boolean;
}
