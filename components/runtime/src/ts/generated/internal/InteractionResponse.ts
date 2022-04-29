import type { IModalCallbackData } from "./IModalCallbackData";
import type { InteractionCallbackData } from "./InteractionCallbackData";

export type InteractionResponse =
  | { kind: "Pong" }
  | { kind: "ChannelMessageWithSource" } & InteractionCallbackData
  | { kind: "DeferredChannelMessageWithSource" } & InteractionCallbackData
  | { kind: "DeferredUpdateMessage" }
  | { kind: "UpdateMessage" } & InteractionCallbackData
  | { kind: "Modal" } & IModalCallbackData;
