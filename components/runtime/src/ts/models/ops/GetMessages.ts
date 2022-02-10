export interface OpGetMessages {
  channelId: string;
  after?: string;
  before?: string;
  limit?: number;
}
