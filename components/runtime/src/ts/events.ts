import { Events, Discord } from './models';

export interface EventTypes {
    /**
     * @internal
     */
    BOTLOADER_COMMAND_INTERACTION_CREATE: Events.CommandInteraction,
    /**
     * @internal
     */
    BOTLOADER_INTERVAL_TIMER_FIRED: Events.IntervalTimerEvent,
    /**
     * @internal
     */
    BOTLOADER_SCHEDULED_TASK_FIRED: Events.ScheduledTask,

    MESSAGE_CREATE: Discord.Message,
    MESSAGE_UPDATE: Events.MessageUpdate,
    MESSAGE_DELETE: Events.MessageDelete,

    MEMBER_ADD: Discord.Member,
    MEMBER_UPDATE: Discord.Member,
    MEMBER_REMOVE: Events.MemberRemove,
}

/**
 * @internal
 */
export namespace InternalEventSystem {

    const eventMuxers: EventMuxer[] = [];

    export function registerEventMuxer(muxer: EventMuxer) {
        eventMuxers.push(muxer)
    }

    export function dispatchEvent(evt: DispatchEvent) {
        let promises: Promise<unknown>[] = [];
        for (let muxer of eventMuxers) {
            promises.push(muxer.handleEvent(evt));
        }
        return Promise.allSettled(promises);
    }
}

BotloaderCore.dispatchEvent = InternalEventSystem.dispatchEvent;

interface DispatchEvent {
    name: string,
    data: any,
}

type ListenerMap = {
    [Property in keyof EventTypes]+?: ((evt: EventTypes[Property]) => void)[];
}

export class EventMuxer {

    listeners: ListenerMap = {};

    /**
     * @internal
     */
    async handleEvent(evt: DispatchEvent) {
        let handlers = this.listeners[evt.name as keyof EventTypes];
        let promises: Promise<unknown>[] = [];
        if (handlers) {
            for (let handler of handlers) {
                let ret = handler(evt.data) as unknown as Promise<unknown>;
                if (ret && 'finally' in ret && 'then' in ret) {
                    let promise = ret as unknown as Promise<unknown>
                    promises.push(promise);
                }
            }
        }

        return PromiseAllSettledHack(promises);
    }

    /**
     * @internal
     */
    on<T extends keyof EventTypes>(eventType: T, cb: (evt: EventTypes[T]) => void) {
        let handlers = this.listeners[eventType];

        // we cast to any since typescript isn't able to handle this
        if (handlers) {
            handlers.push(cb as any);
        } else {
            this.listeners[eventType] = [cb as any];
        }
    }

}

// This is basically my own implementation of Promise.allSettled.
// The problem with Promise.allSettled is that it seems to catch all exceptions
// while this dosen't.
function PromiseAllSettledHack(promises: Promise<unknown>[]) {
    let pending = promises.length;

    if (pending === 0) {
        return Promise.resolve(undefined);
    }

    return new Promise((resolved, _) => {
        for (let promise of promises) {
            promise.finally(() => {
                pending--;
                if (pending === 0) {
                    // all of them resolved
                    resolved(undefined);
                }
            })
        }
    })
}