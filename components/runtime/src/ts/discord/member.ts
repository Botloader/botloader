import { IMember } from "../generated/internal/Member";
import { User } from "./user";

export class Member {
    deaf: boolean;
    joinedAt: number;
    mute: boolean;
    nick: string | null;
    pending: boolean;
    premiumSince: number | null;
    roles: Array<string>;
    user: User;

    constructor(json: IMember) {
        this.deaf = json.deaf;
        this.joinedAt = json.joinedAt;
        this.mute = json.mute;
        this.nick = json.nick;
        this.pending = json.pending;
        this.premiumSince = json.premiumSince;
        this.roles = json.roles;
        this.user = new User(json.user);
    }

    name(): string {
        return this.nick ?? this.user.username;
    }
}


