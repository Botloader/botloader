// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { OpStorageBucketValue } from "./StorageBucketValue";

export interface OpStorageBucketEntry {
  pluginId: string | null;
  bucketName: string;
  key: string;
  value: OpStorageBucketValue;
  expiresAt: number | null;
}
