import { Commands } from './commands';
import {
    ComponentInteraction,
    ModalSubmitInteraction,
    EventMemberRemove,
    EventMessageDelete,
    EventMessageReactionAdd,
    guildChannelFromInternal,
    EventMessageReactionRemove,
    EventMessageReactionRemoveAll,
    IEventThreadDelete,
    EventMessageReactionRemoveAllEmoji,
    EventMessageUpdate,
    Interaction,
    Member,
    Message,
    GuildChannel,
    SelectMenuInteraction,
    parseInteractionCustomId,
    EventInviteCreate,
    EventInviteDelete,
    EventVoiceStateUpdate,
    ThreadMember,
    EventThreadListSync,
    EventThreadMembersUpdate,
    threadChannelFromInternal,
    Thread
} from './discord/index';
import * as Internal from './generated/internal/index';

export namespace EventSystem {

    const buttonComponentListeners: { name: string, cb: (data: ComponentInteraction, extra: any) => any }[] = [];
    const selectMenuListeners: { name: string, cb: (data: SelectMenuInteraction, extra: any) => any }[] = [];
    const modalSubmitListeners: { name: string, cb: (data: ModalSubmitInteraction, extra: any) => any }[] = [];

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
            const converter = converters[evt.name as keyof typeof converters]
            data = converter!(evt.data);
        }

        if (evt.name === "BOTLOADER_COMPONENT_INTERACTION_CREATE") {
            handleComponentInteraction(data);
        } else if (evt.name === "BOTLOADER_COMMAND_INTERACTION_CREATE") {
            commandSystem.handleInteractionCreate(data);
        } else if (evt.name == "BOTLOADER_MODAL_SUBMIT_INTERACTION_CREATE") {
            handleModalSubmitInteraction(data);
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
        BOTLOADER_MODAL_SUBMIT_INTERACTION_CREATE: Internal.IModalInteraction,


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

        VOICE_STATE_UPDATE: EventVoiceStateUpdate,

        MESSAGE_REACTION_ADD: EventMessageReactionAdd,
        MESSAGE_REACTION_REMOVE: EventMessageReactionRemove,
        MESSAGE_REACTION_REMOVE_ALL: EventMessageReactionRemoveAll,
        MESSAGE_REACTION_REMOVE_ALL_EMOJI: EventMessageReactionRemoveAllEmoji,

        CHANNEL_CREATE: GuildChannel,
        CHANNEL_UPDATE: GuildChannel,
        CHANNEL_DELETE: GuildChannel,

        THREAD_CREATE: Thread,
        THREAD_UPDATE: Thread,
        THREAD_DELETE: IEventThreadDelete,
        THREAD_LIST_SYNC: EventThreadListSync,
        THREAD_MEMBER_UPDATE: ThreadMember,
        THREAD_MEMBERS_UPDATE: EventThreadMembersUpdate,

        INVITE_CREATE: EventInviteCreate,
        INVITE_DELETE: EventInviteDelete,
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

    /**
     * @internal
     */
    export function onInteractionModalSubmit<T>(name: string, cb: (interaction: ModalSubmitInteraction, customData: T) => any) {
        modalSubmitListeners.push({ name: name, cb: cb })
    }

    async function handleComponentInteraction(interaction: Internal.MessageComponentInteraction) {
        if (!interaction.customId.startsWith("0:")) {
            return;
        }

        let [name, extras] = parseInteractionCustomId(interaction.customId);

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

    async function handleModalSubmitInteraction(interaction: Internal.IModalInteraction) {
        if (!interaction.customId.startsWith("0:")) {
            return;
        }

        let [name, extras] = parseInteractionCustomId(interaction.customId);

        let listener = modalSubmitListeners.find((elem) => elem.name === name);
        if (listener) {
            let convInteraction = new ModalSubmitInteraction(interaction);
            handleInteractionCallback(convInteraction, async () => {
                await listener!.cb(convInteraction, extras);
            })
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

    const converters: {
        [key in keyof EventTypes]?: (v: any) => EventTypes[key];
    } = {
        MESSAGE_CREATE: (v: Internal.IMessage) => new Message(v),
        MESSAGE_UPDATE: (v: Internal.IEventMessageUpdate) => new EventMessageUpdate(v),

        MESSAGE_REACTION_ADD: (v: Internal.IEventMessageReactionAdd) => new EventMessageReactionAdd(v),

        CHANNEL_CREATE: (v: Internal.InternalGuildChannel) => guildChannelFromInternal(v),
        CHANNEL_UPDATE: (v: Internal.InternalGuildChannel) => guildChannelFromInternal(v),
        CHANNEL_DELETE: (v: Internal.InternalGuildChannel) => guildChannelFromInternal(v),

        MEMBER_ADD: (v: Internal.IMember) => new Member(v),
        MEMBER_UPDATE: (v: Internal.IMember) => new Member(v),
        MEMBER_REMOVE: (v: Internal.IEventMemberRemove) => new EventMemberRemove(v),

        VOICE_STATE_UPDATE: (v: Internal.IEventVoiceStateUpdate) => new EventVoiceStateUpdate(v),

        THREAD_CREATE: (v: Internal.InternalGuildChannel) => threadChannelFromInternal(v),
        THREAD_UPDATE: (v: Internal.InternalGuildChannel) => threadChannelFromInternal(v),

        THREAD_LIST_SYNC: (v: Internal.IEventThreadListSync) => new EventThreadListSync(v),
        THREAD_MEMBER_UPDATE: (v: Internal.IThreadMember) => new ThreadMember(v),
        THREAD_MEMBERS_UPDATE: (v: Internal.IEventThreadMembersUpdate) => new EventThreadMembersUpdate(v),

        INVITE_CREATE: (v: Internal.IEventInviteCreate) => new EventInviteCreate(v),
        INVITE_DELETE: (v: Internal.IEventInviteDelete) => new EventInviteDelete(v),
    }
}
