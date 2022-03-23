import type { ISelectMenuOption } from "./ISelectMenuOption";

export interface ISelectMenu {
  customId: string;
  disabled: boolean;
  minValues?: number;
  maxValues?: number;
  options: Array<ISelectMenuOption>;
  placeholder?: string;
}
