/* eslint-disable @typescript-eslint/naming-convention */
import { User } from "botloader-common";
import { BuildConfig } from "../BuildConfig";
import { debugMessageStore } from "./DebugMessages";
import { sessionManager } from "../util/SessionManager";

export class BotloaderWS {
    ws?: WebSocket;
    baseUrl: string;
    token?: string;
    auth = false;
    onLogMessage: (msg: LogItem) => void;

    subQueue: string[] = [];
    activeSubs: string[] = [];

    constructor(baseUrl: string, onLogMessage: (msg: LogItem) => void, token?: string) {
        this.token = token;
        this.baseUrl = baseUrl;
        this.onLogMessage = onLogMessage;
        if (token) {
            this.open();
        }
    }

    setToken(token?: string) {
        this.logToOutput("updated token", "Client");
        this.token = token;
        if (this.ws) {
            this.ws.onclose = () => { };
            this.ws.close();
        }
        if (token) {
            this.open();
        }
    }

    open() {
        let url = this.baseUrl + "/api/ws";
        this.logToOutput("opening ws to " + url, "Client");
        this.ws = new WebSocket(url);
        this.ws.onopen = this.wsOnOpen.bind(this);
        this.ws.onclose = this.wsOnClose.bind(this);
        this.ws.onmessage = this.wsOnMessage.bind(this);
    }

    send(cmd: WsCommand) {
        if (this.ws) {
            this.ws.send(JSON.stringify(cmd));
        }
    }

    subscribeGuild(guildId: string) {
        if (!this.auth) {
            this.logToOutput("not authorized yet, pushing to queue... " + guildId, "Client");
            this.subQueue.push(guildId);
            return;
        }

        this.logToOutput("subscribing to " + guildId, "Client");
        this.send({
            t: "SubscribeLogs",
            d: guildId,
        });
    }

    sendAuth() {
        this.logToOutput("authorizing ws...", "Client");
        this.send({
            t: "Authorize",
            d: this.token!,
        });
    }

    wsOnOpen() {
        if (this.token) {
            this.sendAuth();
        }
    }

    wsOnMessage(msg: MessageEvent) {
        let decoded: WsEvent = JSON.parse(msg.data.toString());
        switch (decoded.t) {
            case "AuthSuccess":
                this.auth = true;
                this.logToOutput("successfully authorized", "Client");

                for (let g of this.subQueue) {
                    this.subscribeGuild(g);
                }
                this.subQueue = [];

                break;
            case "ScriptLogMessage":
                this.handleScriptLogMessage(decoded);
                break;
            case "SubscriptionsUpdated":
                this.logToOutput("sbuscriptions updated successfully: " + decoded.d, "Client");
                this.activeSubs = decoded.d;
                break;
        }
    }

    wsOnClose(ev: CloseEvent) {
        this.logToOutput(`ws closed ${ev.reason}, reconnecting in 5 sec...`, "Client");
        this.subQueue = this.activeSubs;

        let that = this;
        setTimeout(() => {
            that.open();
        }, 5000);
    }

    handleScriptLogMessage(msg: WsEventScriptLogMessage) {
        console.log("Script log message yoo");
        this.onLogMessage(msg.d);
    }

    logToOutput(msg: string, level: LogLevel, guild_id?: string) {
        this.onLogMessage({
            guild_id,
            level,
            message: msg,
        });
    }
}

type WsEventType = "AuthSuccess" | "SubscriptionsUpdated" | "ScriptLogMessage";

type WsEvent = WsEventAuthorized | WsEventSubscriptionsUpdated | WsEventScriptLogMessage;

interface WsEventAuthorized {
    t: "AuthSuccess",
    d: User,
}

interface WsEventSubscriptionsUpdated {
    t: "SubscriptionsUpdated",
    d: string[],
}


interface WsEventScriptLogMessage {
    t: "ScriptLogMessage",
    d: LogItem,
}

type WsCommandType = "Authorize" | "SubscribeLogs" | "UnSubscribeLogs";

type WsCommand = WsCommandAuthorize | WsCommandSubscribe | WsCommandSubscribe;

interface WsCommandAuthorize {
    t: "Authorize",
    d: string,
}

interface WsCommandSubscribe {
    t: "SubscribeLogs"
    d: string,
}

interface WsCommandUnSubscribe {
    t: "UnSubscribeLogs"
    d: string,
}

export interface LogItem {
    guild_id?: string,
    message: string,
    script_context?: ScriptContext,
    level: LogLevel,
}

export type LogLevel = "Critical" |
    "Error" |
    "Warn" |
    "Info" |
    "ConsoleLog" | "Client";

export interface ScriptContext {
    filename: String,
    line_col?: [number, number],
}

function getWsUrl() {
    if (BuildConfig.botloaderApiBase.startsWith("https")) {
        return BuildConfig.botloaderApiBase.replace("https", "wss")
    } else {
        return BuildConfig.botloaderApiBase.replace("http", "ws")
    }
}

export const WebsocketSession = new BotloaderWS(getWsUrl(), (item) => {
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
});

(() => {
    sessionManager.subscribe((evt) => {
        WebsocketSession.setToken(evt.apiClient.token);
    })

    if (sessionManager.session.apiClient.token) {
        WebsocketSession.setToken(sessionManager.session.apiClient.token);
    }
})()
