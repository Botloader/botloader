import { IComponent } from '../generated/discord/index';
import { encodeInteractionCustomId } from './interaction';

export interface IModalFields {
    title: string,
    customId: string,
    components: IComponent[],
}

export interface IModal extends IModalFields {
    kind: string
}

export class Modal implements IModal {
    kind = "Modal"
    customId: string;
    title: string;
    components: IComponent[];
    
    constructor(title: string, components: IComponent[], name: string, data?: any) {
        this.customId = encodeInteractionCustomId(name, data ?? null)
        this.title = title;
        this.components = components;
    }
}