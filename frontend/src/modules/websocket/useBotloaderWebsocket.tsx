import { useContext } from "react";
import { WebsocketContext } from "./WebsocketContext";

export function useBotloaderWebsocket() {
    const ws = useContext(WebsocketContext)
    return ws
}