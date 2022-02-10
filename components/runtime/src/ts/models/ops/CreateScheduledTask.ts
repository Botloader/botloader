export interface CreateScheduledTask {
  namespace: string;
  uniqueKey?: string;
  data: any;
  executeAt: number;
}
