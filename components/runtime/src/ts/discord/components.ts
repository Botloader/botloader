import type { ButtonStyle } from '../generated/discord/ButtonStyle';
import type { TextInputStyle } from '../generated/discord/TextInputStyle';
import type { ComponentType } from '../generated/discord/ComponentType';
import type { ISelectMenuOption } from '../generated/discord/ISelectMenuOption';
import { ReactionType } from '../generated/discord/ReactionType';
import { ChannelType, IActionRow, IButton, ISelectMenu, ITextInput, SendEmoji } from './index';
import { encodeInteractionCustomId } from './interaction';
import { type SelectMenuType } from '../generated/discord/SelectMenuType';
import { SelectDefaultValue } from '../generated/discord/SelectDefaultValue';

export type AnyComponent = ActionRow | Button | SelectMenu | ShortTextInput | ParagraphTextInput;

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
        super(label, "Primary");

        this.customId = encodeInteractionCustomId(name, data ?? null);
    }

    setStyle(style: ButtonStyle) {
        this.style = style;
        return this;
    }
}

export abstract class BaseSelectMenu extends BaseComponent implements ISelectMenu {
    kind: "SelectMenu" = "SelectMenu";

    customId: string;
    disabled: boolean;
    minValues?: number | undefined;
    maxValues?: number | undefined;
    options: ISelectMenuOption[] = [];
    placeholder?: string | undefined;
    selectType: SelectMenuType;

    constructor(kind: SelectMenuType, name: string, data?: any) {
        super("SelectMenu");

        this.customId = encodeInteractionCustomId(name, data ?? null)
        this.selectType = kind;
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


export class SelectMenu extends BaseSelectMenu {
    constructor(name: string, options: SelectMenuOption[], data?: any) {
        super("Text", name, data);
        this.options = options;
    }
}

export class RoleSelectMenu extends BaseSelectMenu {
    defaultValues?: SelectDefaultValue[];

    constructor(name: string, data?: any) {
        super("Role", name, data);
    }

    setDefaultValues(values: string[]) {
        this.defaultValues = values.map(v => ({ kind: "Role", value: v }))
        return this
    }
}

export class UserSelectMenu extends BaseSelectMenu {
    defaultValues?: SelectDefaultValue[];

    constructor(name: string, data?: any) {
        super("User", name, data);
    }

    setDefaultValues(values: string[]) {
        this.defaultValues = values.map(v => ({ kind: "User", value: v }))
        return this
    }
}

export class ChannelSelectMenu extends BaseSelectMenu {
    defaultValues?: SelectDefaultValue[];
    channelTypes?: ChannelType[];

    constructor(name: string, data?: any) {
        super("Channel", name, data);
    }

    setDefaultValues(values: string[]) {
        this.defaultValues = values.map(v => ({ kind: "Channel", value: v }))
        return this
    }

    setChannelTypes(types: ChannelType[]) {
        this.channelTypes = types
        return this
    }

}

export type MentionableSelectDefaultOption = {
    kind: "User" | "Role",
    value: string,
}

export class MentionableSelectMenu extends BaseSelectMenu {
    defaultValues?: MentionableSelectDefaultOption[];

    constructor(name: string, data?: any) {
        super("Mentionable", name, data);
    }

    /**
     * @example
     * ```ts
     * new MentionableSelectMenu("hello").setDefaultValues([{
     *   kind: "User",
     *   value: "123123213"
     * },{
     *   kind: "Role",
     *   value: "123213132"
     * }])
     * ```
     */
    setDefaultValues(values: MentionableSelectDefaultOption[]) {
        this.defaultValues = values
        return this
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
        this.default = isDefault;
        return this;
    }

    setDescription(description: string) {
        this.description = description;
        return this;
    }
}


export abstract class TextInput extends BaseComponent implements ITextInput {
    kind: "TextInput" = "TextInput";

    customId: string;
    label: string;
    maxLength: number | null = null;
    minLength: number | null = null;
    placeholder: string | null = null;
    required: boolean | null = null;
    style: TextInputStyle;
    value: string | null = null;

    constructor(label: string, style: TextInputStyle, name: string, data?: any) {
        super("TextInput");

        this.label = label;
        this.style = style;
        this.customId = encodeInteractionCustomId(name, data ?? null);
    };

    setMaxLength(length: number) {
        this.maxLength = length;
        return this;
    }
    setMinLength(length: number) {
        this.minLength = length;
        return this;
    }

    setPlaceHolder(placeholder: string) {
        this.placeholder = placeholder;
        return this;
    }

    setValue(value: string) {
        this.value = value;
        return this;
    }

    setRequired(required: boolean) {
        this.required = required;
        return this;
    }
}

export class ShortTextInput extends TextInput implements ITextInput {
    kind: "TextInput" = "TextInput";

    constructor(label: string, name: string, data?: any) {
        super(label, "Short", name, data);
    };
}

export class ParagraphTextInput extends TextInput implements ITextInput {
    kind: "TextInput" = "TextInput";

    constructor(label: string, name: string, data?: any) {
        super(label, "Paragraph", name, data);
    };
}