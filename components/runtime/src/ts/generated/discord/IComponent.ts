import type { IActionRow } from "./IActionRow";
import type { IButton } from "./IButton";
import type { ISelectMenu } from "./ISelectMenu";

export type IComponent =
  | { kind: "ActionRow" } & IActionRow
  | { kind: "Button" } & IButton
  | { kind: "SelectMenu" } & ISelectMenu;
