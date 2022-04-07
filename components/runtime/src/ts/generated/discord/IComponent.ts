import type { IActionRow } from "./IActionRow";
import type { IButton } from "./IButton";
import type { ISelectMenu } from "./ISelectMenu";
import type { ITextInput } from "./ITextInput";

export type IComponent =
  | { kind: "ActionRow" } & IActionRow
  | { kind: "Button" } & IButton
  | { kind: "SelectMenu" } & ISelectMenu
  | { kind: "TextInput" } & ITextInput;
