import type { CommandSubGroup } from "./CommandSubGroup";

export interface CommandGroup {
  name: string;
  description: string;
  subGroups: Array<CommandSubGroup>;
}
