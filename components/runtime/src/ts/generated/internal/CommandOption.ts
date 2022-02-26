import type { CommandOptionType } from "./CommandOptionType";
import type { ExtraCommandOptions } from "./ExtraCommandOptions";

export interface CommandOption {
  name: string;
  description: string;
  kind: CommandOptionType;
  required: boolean;
  extraOptions: ExtraCommandOptions;
}
