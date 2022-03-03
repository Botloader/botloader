import type { SelectMenuOption } from "./SelectMenuOption";

export interface SelectMenu {
  customId: string;
  disabled: boolean;
  minValues?: number;
  maxValues?: number;
  options: Array<SelectMenuOption>;
  placeholder?: string;
}
