import type { IInviteChannel } from "../generated/discord/IInviteChannel";
import type { IInviteGuild } from "../generated/discord/IInviteGuild";
import type { InviteTargetType } from "../generated/discord/InviteTargetType";
import type { IInvite } from "../generated/internal/IInvite";
import { User } from "./user";

export class Invite {
    approximateMemberCount?: number;
    approximatePresenceCount?: number;
    channel: IInviteChannel | null;
    code: string;
    createdAt?: number;
    expiresAt?: number;
    guild?: IInviteGuild;
    inviter?: User;
    maxAgeSeconds?: number;
    maxUses?: number;
    targetType?: InviteTargetType;
    targetUser?: User;
    temporary?: boolean;
    uses?: number;

    /** 
    * @internal 
    */
    constructor(json: IInvite) {
        this.approximateMemberCount = json.approximateMemberCount
        this.approximatePresenceCount = json.approximatePresenceCount
        this.channel = json.channel
        this.code = json.code
        this.createdAt = json.createdAt
        this.expiresAt = json.expiresAt
        this.guild = json.guild
        this.inviter = json.inviter && new User(json.inviter)
        this.maxAgeSeconds = json.maxAge
        this.maxUses = json.maxUses
        this.targetType = json.targetType
        this.targetUser = json.targetUser && new User(json.targetUser)
        this.temporary = json.temporary
        this.uses = json.uses
    }
}