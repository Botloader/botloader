import { ButtonStyle } from '../generated/discord/ButtonStyle';
import type { ComponentType } from '../generated/discord/ComponentType';
import { ISelectMenuOption } from '../generated/discord/ISelectMenuOption';
import { ReactionType } from '../generated/discord/ReactionType';
import { IActionRow, IButton, ISelectMenu, SendEmoji } from './index';
import { encodeInteractionCustomId } from './interaction';

export type AnyComponent = ActionRow | Button | SelectMenu;

export abstract class BaseComponent {
    kind: ComponentType;

    constructor(kind: ComponentType) {
        this.kind = kind;
    }
}

export class ActionRow extends BaseComponent implements IActionRow {
    kind: "ActionRow" = "ActionRow";
    components: AnyComponent[];


    constructor(children: AnyComponent[]) {
        super("ActionRow");

        this.components = children;
    }
}

export abstract class Button extends BaseComponent implements IButton {
    kind: "Button" = "Button";

    customId?: string;
    style: ButtonStyle;
    disabled?: boolean;
    url?: string;
    label?: string;
    emoji?: ReactionType;

    constructor(label: string, style: ButtonStyle) {
        super("Button");

        this.label = label;
        this.style = style;
    };

    setDisabled(disabled: boolean) {
        this.disabled = disabled;
        return this;
    }

    setEmoji(emoji: SendEmoji) {
        if ("unicode" in emoji) {
            this.emoji = emoji;
        } else {
            this.emoji = {
                id: emoji.id,
                name: emoji.name ?? null,
                animated: false,
            };
        }


        return this;
    }
}

export class UrlButton extends Button implements IButton {
    constructor(label: string, url: string) {
        super(label, "Link");

        this.url = url;
    }
}

export class CustomButton extends Button implements IButton {
    constructor(label: string, name: string, data?: any) {
        super(label, "Link");

        this.customId = encodeInteractionCustomId(name, data ?? null);
    }

    setStyle(style: ButtonStyle) {
        this.style = style;
        return this;
    }
}

export class SelectMenu extends BaseComponent implements ISelectMenu {
    kind: "SelectMenu" = "SelectMenu";

    customId: string;
    disabled: boolean;
    minValues?: number | undefined;
    maxValues?: number | undefined;
    options: ISelectMenuOption[];
    placeholder?: string | undefined;

    constructor(name: string, options: SelectMenuOption[], data?: any) {
        super("SelectMenu");

        this.customId = encodeInteractionCustomId(name, data ?? null)
        this.options = options;
        this.disabled = false;
    }

    setDisabled(disabled: boolean) {
        this.disabled = disabled;
        return this;
    }

    setMinValues(minValues: number) {
        this.minValues = minValues;
        return this;
    }

    setMaxValues(maxValues: number) {
        this.maxValues = maxValues;
        return this;
    }

    setPlaceHolder(placeholder: string) {
        this.placeholder = placeholder;
        return this;
    }
}

export class SelectMenuOption implements ISelectMenuOption {
    default: boolean;
    description?: string | undefined;
    emoji?: ReactionType | undefined;
    label: string;
    value: string;

    constructor(label: string, value: string) {
        this.label = label;
        this.value = value;

        this.default = false;
    }

    setEmoji(emoji: SendEmoji) {
        if ("unicode" in emoji) {
            this.emoji = emoji;
        } else {
            this.emoji = {
                id: emoji.id,
                name: emoji.name ?? null,
                animated: false,
            };
        }

        return this;
    }

    setDefault(isDefault: boolean) {
        this.default = this.default;
        return this;
    }

    setDescription(description: string) {
        this.description = description;
        return this;
    }
} 