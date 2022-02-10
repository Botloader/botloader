export interface ScheduledTask {
  id: number;
  namespace: string;
  key?: string;
  executeAt: number;
  data: unknown;
}
