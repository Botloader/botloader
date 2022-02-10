export interface ClientHttpRequest {
  path: string;
  method: string;
  headers: Record<string, string>;
  scriptId?: number;
  bodyResourceId?: number;
}
