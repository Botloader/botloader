// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IInviteChannel } from "../discord/IInviteChannel";
import type { IInviteGuild } from "../discord/IInviteGuild";
import type { IUser } from "./IUser";
import type { InviteTargetType } from "../discord/InviteTargetType";

export interface IInvite {
  approximateMemberCount?: number;
  approximatePresenceCount?: number;
  channel: IInviteChannel | null;
  code: string;
  createdAt?: number;
  expiresAt?: number;
  guild?: IInviteGuild;
  inviter?: IUser;
  maxAge?: number;
  maxUses?: number;
  targetType?: InviteTargetType;
  targetUser?: IUser;
  temporary?: boolean;
  uses?: number;
}
