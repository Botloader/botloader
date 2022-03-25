import type { IUser } from "../generated/internal/IUser";
import type { CdnImageSize } from "./common";

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

    /**
     * @internal
     */
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

    /**
     * @returns a url to the user's avatar with the desired size (defaults to 256) 
     */
    avatarUrl(options?: { size?: CdnImageSize }): string {
        const base = "https://cdn.discordapp.com/"
        const size = options?.size ?? 128;


        if (this.avatar) {
            let format = this.avatar.startsWith("a_") ? "gif" : "png";
            return base + `avatars/${this.id}/${this.avatar}.${format}?size=${size}`
        }

        const parsedDiscrim = parseInt(this.discriminator);
        return base + `embed/avatars/${parsedDiscrim % 5}.png?size=${size}`
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
