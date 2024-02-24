// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Command } from "./Command";
import type { CommandGroup } from "./CommandGroup";
import type { IntervalTimer } from "./IntervalTimer";
import type { SettingsOptionDefinition } from "./SettingOptionDefinition";
import type { TaskBucketId } from "./ScriptTaskBucketId";

export interface ScriptMeta {
  description: string;
  scriptId: number;
  pluginId: string | null;
  commands: Array<Command>;
  commandGroups: Array<CommandGroup>;
  intervalTimers: Array<IntervalTimer>;
  taskBuckets: Array<TaskBucketId>;
  settings: Array<SettingsOptionDefinition>;
}
