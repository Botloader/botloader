import { IUser } from "../generated/internal/IUser";

export class User {
    avatar: string | null;
    bot: boolean;
    discriminator: string;
    id: string;
    locale: string | null;
    username: string;
    premiumType: PremiumType | null;
    publicFlags: UserFlags | null;
    system: boolean | null;

    constructor(json: IUser) {
        this.avatar = json.avatar;
        this.bot = json.bot;
        this.discriminator = json.discriminator;
        this.id = json.id;
        this.locale = json.locale;
        this.username = json.username;
        this.premiumType = json.premiumType;
        this.publicFlags = json.publicFlags;
        this.system = json.system;
    }

    mention() {
        return `<@${this.id}>`;
    }
}

export interface UserFlags {
    staff: boolean;
    partner: boolean;
    hypesquad: boolean;
    bugHunterLevel1: boolean;
    hypesquadOnlineHouse1: boolean;
    hypesquadOnlineHouse2: boolean;
    hypesquadOnlineHouse3: boolean;
    premiumEarlySupporter: boolean;
    teamPseudoUser: boolean;
    bugHunterLevel2: boolean;
    verifiedBot: boolean;
    verifiedDeveloper: boolean;
    certifiedModerator: boolean;
    botHttpInteractions: boolean;
}


export type PremiumType = "none" | "nitroClassic" | "nitro";
