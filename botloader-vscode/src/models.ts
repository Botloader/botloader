import { UserGuild } from "botloader-common";

export interface IndexFile {
    guild: UserGuild,
    openScripts: IndexScript[],
}

export interface IndexScript {
    id: number,
    name: string,
}