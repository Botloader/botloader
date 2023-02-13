export namespace GuildMessages {
    const messageBuffers: MessageBuffers = {}

    const listeners: Listeners = {}
    let currentId = 1;
    let currentMessageId = 1;

    export function pushGuildMessage(guildId: string, msg: CreateGuildMessage) {
        let fullMessage = {
            ...msg,
            id: currentMessageId++,
        }

        if (!messageBuffers[guildId]) {
            messageBuffers[guildId] = [];
        }

        messageBuffers[guildId].push(fullMessage);
        // keep max 100 entries in the buffer
        if (messageBuffers[guildId].length > 100) {
            messageBuffers[guildId] = [
                ...messageBuffers[guildId].slice(10)
            ]
        }

        if (listeners[guildId]) {
            for (let listener of listeners[guildId]) {
                listener.cb(fullMessage);
            }
        }
    }


    export function addListener(guildId: string, listener: (msg: GuildMessage) => void) {
        let id = currentId++;

        if (!listeners[guildId]) {
            listeners[guildId] = [];
        }
        listeners[guildId].push({
            id: id,
            cb: listener,
        })

        return id
    }

    export function removeListener(guildId: string, id: number) {
        if (listeners[guildId]) {
            let index = listeners[guildId].findIndex(v => v.id === id);
            if (index > -1) {
                listeners[guildId].splice(index, 1);
            }
        }
    }

    export function getGuildMessages(guildId: string): GuildMessage[] {
        return messageBuffers[guildId] ?? []
    }

    interface MessageBuffers {
        [key: string]: GuildMessage[]
    }

    interface Listeners {
        [key: string]: Listener[],
    }

    interface Listener {
        id: number,
        cb: (msg: GuildMessage) => void
    }
}

export type MessageLevel = "Critical" |
    "Error" |
    "Warn" |
    "Info" |
    "ConsoleLog" | "Client";

export interface CreateGuildMessage {
    level: MessageLevel,
    context?: string,
    message: string,
}

export interface GuildMessage {
    id: number,
    level: MessageLevel,
    context?: string,
    message: string,
}
