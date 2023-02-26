class DebugMessageStore {
    messageBuffers: MessageBuffers = {}

    listeners: Listeners = {}
    currentId = 1;
    currentMessageId = 1;

    pushMessage(msg: CreateDebugMessage) {
        const fullMessage = {
            ...msg,
            id: this.currentMessageId++,
        }

        const key = msg.guildId ? msg.guildId : "global";

        if (!this.messageBuffers[key]) {
            this.messageBuffers[key] = [];
        }

        this.messageBuffers[key].push(fullMessage);

        // keep max 100 entries in the buffer
        if (this.messageBuffers[key].length > 100) {
            this.messageBuffers[key] = [
                ...this.messageBuffers[key].slice(10)
            ]
        }

        if (key === "global") {
            // global triggers all guild listeners
            for (let [, table] of Object.entries(this.listeners)) {
                for (let [, listener] of Object.entries(table)) {
                    listener.cb(fullMessage);
                }
            }
        } else {
            // only trigger listeners for this guild
            if (this.listeners[key]) {
                for (let [, listener] of Object.entries(this.listeners[key])) {
                    listener.cb(fullMessage);
                }
            }
        }
    }

    addListener(key: string, listener: (msg: DebugMessage) => void) {
        const id = this.currentId++;


        if (!this.listeners[key]) {
            this.listeners[key] = {};
        }

        this.listeners[key][id + ""] = {
            id: id + "",
            cb: listener,
        };

        return id
    }

    removeListener(bufferKey: string, id: number) {
        if (this.listeners[bufferKey]) {
            delete this.listeners[bufferKey][id + ""];
        }
    }

    getGuildMessages(guildId: string): DebugMessage[] {
        return this.messageBuffers[guildId] ?? []
    }

}

export const debugMessageStore = new DebugMessageStore();

interface MessageBuffers {
    [key: string]: DebugMessage[]
}

interface Listeners {
    [key: string]: ListernerIdMap,
}

interface ListernerIdMap {
    [key: string]: Listener,
}

interface Listener {
    id: string,
    cb: (msg: DebugMessage) => void
}

export type MessageLevel = "Critical" |
    "Error" |
    "Warn" |
    "Info" |
    "ConsoleLog" | "Client";

export type CreateDebugMessage = Omit<DebugMessage, "id">;

export interface DebugMessage {
    id: number,
    level: MessageLevel,
    context?: string,
    message: string,
    guildId?: string,
}
