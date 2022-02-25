import type { Command } from "./Command";
import type { CommandGroup } from "./CommandGroup";
import type { IntervalTimer } from "./IntervalTimer";

export interface ScriptMeta {
  description: string;
  scriptId: number;
  commands: Array<Command>;
  commandGroups: Array<CommandGroup>;
  intervalTimers: Array<IntervalTimer>;
  taskNames: Array<string>;
}
