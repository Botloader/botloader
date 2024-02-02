import { createContext, useEffect, useState } from "react";
import { BotloaderWS, getWsUrl } from "./WebsocketController";
import { useSession } from "../session/useSession";
import { debugMessageStore } from "../../misc/DebugMessages";

export const WebsocketContext = createContext<BotloaderWS>(new BotloaderWS(getWsUrl(), () => { }, null));

export function BotloaderWebsocketProvider({ children }: { children: React.ReactNode }) {
    const session = useSession()
    const [ws, setWs] = useState(new BotloaderWS(getWsUrl(), () => { }, null))

    useEffect(() => {
        setWs(current => {
            if (current.token === session.apiClient.token) {
                return current
            }

            current.close()

            return new BotloaderWS(getWsUrl(), (item) => {
                let context = "";
                if (item.script_context) {
                    context += ` ${item.script_context.filename}`;
                    if (item.script_context.line_col) {
                        const [line, col] = item.script_context.line_col;
                        context += `:${line}:${col}`;
                    }
                }

                console.log(`[WS]:[${item.level} ${context}] ${item.message}`)
                debugMessageStore.pushMessage({
                    guildId: item.guild_id,
                    level: item.level,
                    context: context,
                    message: item.message,
                })
            }, session.apiClient.token ?? null)
        })
    }, [session])

    return <WebsocketContext.Provider value={ws}>{children}</WebsocketContext.Provider>
}

