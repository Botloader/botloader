export interface ClientHttpResponse {
  headers: Record<string, string>;
  statusCode: number;
  bodyResourceId: number;
}
