import { IMember } from "../generated/internal/Member";
import { IBan } from "../generated/internal/Ban";
import { User } from "./user";

export class Member {
    deaf: boolean;
    joinedAt: number;
    mute: boolean;
    nick: string | null;
    pending: boolean;
    premiumSince: number | null;
    roles: Array<string>;
    communicationDisabledUntil: number | null;
    user: User;

    /**
     * @internal
     */
    constructor(json: IMember) {
        this.deaf = json.deaf;
        this.joinedAt = json.joinedAt;
        this.mute = json.mute;
        this.nick = json.nick;
        this.pending = json.pending;
        this.premiumSince = json.premiumSince;
        this.roles = json.roles;
        this.communicationDisabledUntil = json.communicationDisabledUntil;
        this.user = new User(json.user);
    }

    name(): string {
        return this.nick ?? this.user.username;
    }
}

export class Ban {
    reason: string | null;
    user: User;

    /**
     * @internal
     */
    constructor(json: IBan) {
        this.reason = json.reason;
        this.user = new User(json.user);
    }
}