import { Events, Internal } from './generated';
import * as Discord from './generated/discord/index';

export interface EventTypes {
    /**
     * @internal
     */
    BOTLOADER_COMMAND_INTERACTION_CREATE: Internal.CommandInteraction,
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
        for (let muxer of eventMuxers) {
            muxer.handleEvent(evt)
        }
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

        if (handlers) {
            for (let handler of handlers) {
                handler(evt.data);
            }
        }
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