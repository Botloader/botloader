import type { IntervalTimer } from "./IntervalTimer";
import type { CommandGroup } from "./CommandGroup";
import type { Command } from "./Command";

export interface ScriptMeta {
  description: string;
  scriptId: number;
  commands: Array<Command>;
  commandGroups: Array<CommandGroup>;
  intervalTimers: Array<IntervalTimer>;
  taskNames: Array<string>;
}
