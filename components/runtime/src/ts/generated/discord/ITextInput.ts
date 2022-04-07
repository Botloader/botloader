import type { TextInputStyle } from "./TextInputStyle";

export interface ITextInput {
  customId: string;
  label: string;
  maxLength: number | null;
  minLength: number | null;
  placeholder: string | null;
  required: boolean | null;
  style: TextInputStyle;
  value: string | null;
}
