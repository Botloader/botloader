export interface LogMessage {
  fileName?: string;
  lineNumber?: number;
  colNumber?: number;
  message: string;
}
