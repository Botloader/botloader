import type { IUser } from "./IUser";
import type { PartialMember } from "../discord/PartialMember";

export interface IUserMention {
  user: IUser;
  member: PartialMember | null;
}
