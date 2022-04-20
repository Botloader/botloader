import type { IComponent } from "../discord/IComponent";

export interface IModalCallbackData {
  title: string;
  customId: string;
  components: Array<IComponent>;
}
