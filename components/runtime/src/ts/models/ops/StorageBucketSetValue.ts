import type { OpStorageBucketValue } from "./StorageBucketValue";

export interface OpStorageBucketSetValue {
  bucketName: string;
  key: string;
  value: OpStorageBucketValue;
  ttl?: number;
}
