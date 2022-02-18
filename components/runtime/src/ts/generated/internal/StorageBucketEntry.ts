import type { OpStorageBucketValue } from "./StorageBucketValue";

export interface OpStorageBucketEntry {
  bucketName: string;
  key: string;
  value: OpStorageBucketValue;
  expiresAt: number | null;
}
