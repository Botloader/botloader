import type { SendEmoji } from "../discord/SendEmoji";

export interface GetReactionsFields {
  emoji: SendEmoji;
  after?: string;
  limit?: number;
}
