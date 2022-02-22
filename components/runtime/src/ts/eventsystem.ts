import { Internal } from './generated';
import * as Discord from './generated/discord/index';

export namespace EventSystem {

    const eventMuxers: Muxer[] = [];

    /**
     * @internal
     */
    export function registerEventMuxer(muxer: Muxer) {
        eventMuxers.push(muxer)
    }

    /**
     * @internal
     */
    export function dispatchEvent(evt: DispatchEvent) {
        for (let muxer of eventMuxers) {
            muxer.handleEvent(evt)
        }
    }
    BotloaderCore.dispatchEvent = dispatchEvent;

    export interface EventTypes {
        /**
         * @internal
         */
        BOTLOADER_COMMAND_INTERACTION_CREATE: Internal.CommandInteraction,
        /**
         * @internal
         */
        BOTLOADER_INTERVAL_TIMER_FIRED: Internal.IntervalTimerEvent,
        /**
         * @internal
         */
        BOTLOADER_SCHEDULED_TASK_FIRED: Internal.ScheduledTask,

        MESSAGE_CREATE: Discord.Message,
        MESSAGE_UPDATE: Discord.EventMessageUpdate,
        MESSAGE_DELETE: Discord.EventMessageDelete,

        MEMBER_ADD: Discord.Member,
        MEMBER_UPDATE: Discord.Member,
        MEMBER_REMOVE: Discord.EventMemberRemove,
    }


    interface DispatchEvent {
        name: string,
        data: any,
    }

    type ListenerMap = {
        [Property in keyof EventTypes]+?: ((evt: EventTypes[Property]) => void)[];
    }

    /**
     * @internal
     */
    export class Muxer {

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
}