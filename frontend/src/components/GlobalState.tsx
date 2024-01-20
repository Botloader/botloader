import { BotGuild } from "botloader-common"
import { Session } from "../modules/session/SessionContext"

export type GlobalState = {
    session: Session,
    guilds: BotGuild[],
}