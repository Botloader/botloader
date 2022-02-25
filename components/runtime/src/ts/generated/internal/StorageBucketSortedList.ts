import type { OpStorageBucketListOrder } from "./StorageBucketListOrder";

export interface OpStorageBucketSortedList {
  bucketName: string;
  offset?: number;
  limit?: number;
  order: OpStorageBucketListOrder;
}
