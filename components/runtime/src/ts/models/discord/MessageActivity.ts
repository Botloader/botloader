import type { MessageActivityType } from "./MessageActivityType";

export interface MessageActivity {
  kind: MessageActivityType;
  partyId: string | null;
}
