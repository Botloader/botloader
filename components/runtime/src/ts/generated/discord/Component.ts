import type { ActionRow } from "./ActionRow";
import type { Button } from "./Button";
import type { SelectMenu } from "./SelectMenu";

export type Component =
  | { kind: "ActionRow" } & ActionRow
  | { kind: "Button" } & Button
  | { kind: "SelectMenu" } & SelectMenu;
