import type { ComponentType } from "../discord/ComponentType";

export interface IModalInteractionDataComponent {
  customId: string;
  kind: ComponentType;
  value: string;
}
