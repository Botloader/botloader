import type { OpStorageBucketValue } from "./StorageBucketValue";
import type { OpStorageBucketSetCondition } from "./StorageBucketSetCondition";

export interface OpStorageBucketSetIf {
  bucketName: string;
  key: string;
  value: OpStorageBucketValue;
  ttl?: number;
  cond: OpStorageBucketSetCondition;
}
