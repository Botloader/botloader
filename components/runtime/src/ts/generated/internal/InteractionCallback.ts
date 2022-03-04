import type { InteractionResponse } from "./InteractionResponse";

export interface InteractionCallback {
  interactionId: string;
  ineractionToken: string;
  data: InteractionResponse;
}
