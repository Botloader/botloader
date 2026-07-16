import { CurrentUserGuild } from "botloader-common";

export interface IndexFile {
    guild: CurrentUserGuild,
    openScripts: IndexScript[],
}

export interface IndexScript {
    id: number,
    name: string,
}