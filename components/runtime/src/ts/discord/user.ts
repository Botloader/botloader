import type { IUser } from "../generated/internal/IUser";
import type { CdnImageSize } from "./common";
import type { PremiumType } from "../generated/internal/PremiumType"
export type { PremiumType } from "../generated/internal/PremiumType"

export class User {
    avatar: string | null;
    bot: boolean;
    discriminator: string;
    id: string;
    locale: string | null;

    username: string;

    /**
     * Global nickname
     */
    globalName: string | null;

    premiumType: PremiumType | null;
    publicFlags: UserFlags | null;
    system: boolean | null;

    /**
     * Banner hash
     */
    banner: string | null;

    /**
     * The user's banner color encoded as an integer representation of hexadecimal color code
     * 
     * For example: 0xff0000 for red
     */
    accentColor: number | null;


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
        this.accentColor = json.accentColor;
        this.globalName = json.globalName;
        this.banner = json.banner;
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

    /**
     * @returns a timestamp for when the user was created
     */
    createdAt(): Date {
        const snowflake = BigInt(this.id);
        const unixTime = (snowflake >> 22n) + 1420070400000n
        return new Date(Number(unixTime));
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

// export type PremiumType = "none" | "nitroClassic" | "nitro" | "nitroBasic";
