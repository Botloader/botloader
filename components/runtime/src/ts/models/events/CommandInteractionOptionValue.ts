export type CommandInteractionOptionValue =
  | { kind: "string"; value: string }
  | { kind: "integer"; value: bigint }
  | { kind: "boolean"; value: boolean }
  | { kind: "user"; value: string }
  | { kind: "channel"; value: string }
  | { kind: "role"; value: string }
  | { kind: "mentionable"; value: string }
  | { kind: "number"; value: number };
