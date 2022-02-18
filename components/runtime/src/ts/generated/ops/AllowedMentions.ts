import type { MentionParseTypes } from "./MentionParseTypes";

export interface AllowedMentions {
  parse: Array<MentionParseTypes>;
  users: Array<string>;
  roles: Array<string>;
  repliedUser: boolean;
}
