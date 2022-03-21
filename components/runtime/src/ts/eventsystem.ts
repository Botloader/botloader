import { Commands } from './commands';
import { ComponentInteraction, EventMemberRemove, EventMessageDelete, EventMessageReactionAdd, EventMessageReactionRemove, EventMessageReactionRemoveAll, EventMessageReactionRemoveAllEmoji, EventMessageUpdate, Interaction, Member, Message, SelectMenuInteraction } from './discord/index';
import * as Internal from './generated/internal/index';

export namespace EventSystem {

    const buttonComponentListeners: { name: string, cb: (data: ComponentInteraction, extra: any) => any }[] = [];
    const selectMenuListeners: { name: string, cb: (data: SelectMenuInteraction, extra: any) => any }[] = [];

    /**
     * @internal
     */
    export const commandSystem = new Commands.System();

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
        let data = evt.data;
        if (evt.name in converters) {
            data = converters[evt.name as keyof typeof converters](evt.data);
        }

        if (evt.name === "BOTLOADER_COMPONENT_INTERACTION_CREATE") {
            handleComponentInteraction(data);
        } else if (evt.name === "BOTLOADER_COMMAND_INTERACTION_CREATE") {
            commandSystem.handleInteractionCreate(data)
        } else {
            for (let muxer of eventMuxers) {
                muxer.handleEvent(evt.name, data);
            }
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
        BOTLOADER_COMPONENT_INTERACTION_CREATE: Internal.MessageComponentInteraction,


        /**
         * @internal
         */
        BOTLOADER_INTERVAL_TIMER_FIRED: Internal.IntervalTimerEvent,
        /**
         * @internal
         */
        BOTLOADER_SCHEDULED_TASK_FIRED: Internal.ScheduledTask,

        MESSAGE_CREATE: Message,
        MESSAGE_UPDATE: EventMessageUpdate,
        MESSAGE_DELETE: EventMessageDelete,

        MEMBER_ADD: Member,
        MEMBER_UPDATE: Member,
        MEMBER_REMOVE: EventMemberRemove,

        MESSAGE_REACTION_ADD: EventMessageReactionAdd,
        MESSAGE_REACTION_REMOVE: EventMessageReactionRemove,
        MESSAGE_REACTION_REMOVE_ALL: EventMessageReactionRemoveAll,
        MESSAGE_REACTION_REMOVE_ALL_EMOJI: EventMessageReactionRemoveAllEmoji,
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
        async handleEvent(name: string, data: any) {
            let handlers = this.listeners[name as keyof EventTypes];

            if (handlers) {
                for (let handler of handlers) {
                    handler(data);
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

    /**
     * @internal
     */
    export function onInteractionButton<T>(name: string, cb: (interaction: ComponentInteraction, extraData: T) => any) {
        buttonComponentListeners.push({ name: name, cb: cb })
    }
    /**
     * @internal
     */
    export function onInteractionSelectMenu<T>(name: string, cb: (interaction: SelectMenuInteraction, extraData: T) => any) {
        selectMenuListeners.push({ name: name, cb: cb })
    }

    async function handleComponentInteraction(interaction: Internal.MessageComponentInteraction) {
        if (!interaction.customId.startsWith("0:")) {
            return;
        }

        let customId = interaction.customId.slice(2);
        let nameEnd = customId.indexOf(":");
        let name = "";
        let extras: unknown = null;

        if (nameEnd > -1) {
            name = customId.slice(0, nameEnd)
            let extrasStr = customId.slice(nameEnd + 1);
            if (extrasStr) {
                extras = JSON.parse(extrasStr);
            }
        }

        if (interaction.componentType === "Button") {
            let listener = buttonComponentListeners.find((elem) => elem.name === name);
            if (listener) {
                let convInteraction = new ComponentInteraction(interaction);
                handleInteractionCallback(convInteraction, async () => {
                    await listener!.cb(convInteraction, extras);
                })
            }
        } else if (interaction.componentType === "SelectMenu") {
            let listener = selectMenuListeners.find((elem) => elem.name === name);
            if (listener) {
                let convInteraction = new SelectMenuInteraction(interaction);
                handleInteractionCallback(convInteraction, async () => {
                    await listener!.cb(convInteraction, extras);
                })
            }
        }
    }

    async function handleInteractionCallback(interaction: Interaction, inner: () => any) {
        try {
            await inner();
        } catch (e) {
            if (!interaction.hasSentCallback) {
                await interaction.ackWithMessage({
                    content: "An error occured handling the interaction: " + e,
                    flags: { ephemeral: true },
                })
            } else {
                await interaction.sendFollowup({ content: "An error occured handling the interaction: " + e, flags: { ephemeral: true } })
            }
        } finally {
            // send no response message if needed
            if (!interaction.hasSentCallback) {
                await interaction.ackWithMessage({
                    content: "No response for interaction, this is probably a bug in the script",
                    flags: { ephemeral: true },
                })
            }
        }
    }

    const converters = {
        MESSAGE_CREATE: (v: Internal.IMessage) => new Message(v),
    }
}
