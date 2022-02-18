import type { OpStorageBucketSetCondition } from "./StorageBucketSetCondition";
import type { OpStorageBucketValue } from "./StorageBucketValue";

export interface OpStorageBucketSetIf {
  bucketName: string;
  key: string;
  value: OpStorageBucketValue;
  ttl?: number;
  cond: OpStorageBucketSetCondition;
}
