import type { ButtonStyle } from '../generated/discord/ButtonStyle';
import type { TextInputStyle } from '../generated/discord/TextInputStyle';
import type { ComponentType } from '../generated/discord/ComponentType';
import type { ISelectMenuOption } from '../generated/discord/ISelectMenuOption';
import type {ICheckboxGroupOption} from '../generated/discord/ICheckboxGroupOption';
import { ReactionType } from '../generated/discord/ReactionType';
import {
    ChannelType,
    IActionRow,
    IButton,
    ISelectMenu,
    ITextInput,
    ISection,
    ITextDisplay,
    IThumbnail,
    IMediaGallery,
    IFile,
    ISeparator,
    IContainer,
    ILabel,
    IFileUpload,
    //IUnknownComponent,
    SendEmoji, 
    //IComponent,
    IUnfurledMediaItem,
    IMediaGalleryItem,
    SeparatorSpacingSize,
    ICheckbox,
    ICheckboxGroup,
} from './index';
import { encodeInteractionCustomId } from './interaction';
import { type SelectMenuType } from '../generated/discord/SelectMenuType';
import { SelectDefaultValue } from '../generated/discord/SelectDefaultValue';

export type AnyComponent = 
    | ActionRow
    | Button
    | SelectMenu
    | ShortTextInput
    | ParagraphTextInput
    | Section
    | TextDisplay
    | Thumbnail
    | MediaGallery
    | File
    | Separator
    | Container
    | Label
    | FileUpload
    | Checkbox
    | CheckboxGroup;

export abstract class BaseComponent {
    kind: ComponentType;
    id?: number | undefined;

    constructor(kind: ComponentType) {
        this.kind = kind;
    }

    setId(id: number) {
        this.id = id;
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

    constructor(style: ButtonStyle) {
        super("Button");
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

    setLabel(label: string) {
        this.label = label;
        return this;
    }
}

export class UrlButton extends Button implements IButton {
    constructor(label: string | null, url: string) {
        super("Link");
        if (label) {
            this.label = label;
        }

        this.url = url;
    }
}

export class CustomButton extends Button implements IButton {
    constructor(label: string | null, name: string, data?: any) {
        super("Primary");
        if (label) {
            this.label = label;
        }

        this.customId = encodeInteractionCustomId(name, data ?? null);
    };

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
    required?: boolean;
    selectType: SelectMenuType;

    constructor(kind: SelectMenuType, name: string, data?: any) {
        super("SelectMenu");

        this.customId = encodeInteractionCustomId(name, data ?? null)
        this.selectType = kind;
        this.disabled = false;
    }

    setRequired(required: boolean) {
        this.required = required;
        return this;
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
    /** @deprecated */
    label?: string;
    maxLength?: number;
    minLength?: number;
    placeholder?: string;
    required?: boolean;
    style: TextInputStyle;
    value?: string;

    constructor(style: TextInputStyle, name: string, data?: any) {
        super("TextInput");

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

    constructor(label: string | null, name: string, data?: any) {
        super("Short", name, data);
        if (label) {
            this.label = label;
        }
    };
}

export class ParagraphTextInput extends TextInput implements ITextInput {
    kind: "TextInput" = "TextInput";

    constructor(label: string | null, name: string, data?: any) {
        super("Paragraph", name, data);
        if (label) {
            this.label = label;
        }
    };
}

export class Section extends BaseComponent implements ISection {
    kind: "Section" = "Section";

    components: AnyComponent[];
    accessory: AnyComponent;

    constructor(children: AnyComponent[], accessory: AnyComponent) {
        super("Section");
        
        this.components = children;
        this.accessory = accessory;
    }
}

export class TextDisplay extends BaseComponent implements ITextDisplay {
    kind: "TextDisplay" = "TextDisplay";

    content: string;

    constructor(content: string) {
        super("TextDisplay");

        this.content = content;
    }
}

export class UnfurledMediaItem implements IUnfurledMediaItem {
    url: string;
    proxyUrl?: string;
    height?: number;
    width?: number;
    contentType?: string;

    constructor (url: string) {
        this.url = url;
    }
}

export class Thumbnail extends BaseComponent implements IThumbnail {
    kind: "Thumbnail" = "Thumbnail";

    media: UnfurledMediaItem;
    description?: string;
    spoiler?: boolean;

    constructor(media: UnfurledMediaItem) {
        super("Thumbnail");

        this.media = media;
    }

    setDescription(description: string) {
        this.description = description;
        return this;
    }

    setSpoiler(spoiler: boolean) {
        this.spoiler = spoiler;
        return this;
    }
}

export class MediaGalleryItem implements IMediaGalleryItem {
    media: UnfurledMediaItem;
    description?: string;
    spoiler?: boolean;
    
    constructor(media: UnfurledMediaItem) {
        this.media = media;
    }

    setDescription(description: string) {
        this.description = description;
        return this;
    }

    setSpoiler(spoiler: boolean) {
        this.spoiler = spoiler;
        return this;
    }
}

export class MediaGallery extends BaseComponent implements IMediaGallery {
    kind: "MediaGallery" = "MediaGallery";
    
    items: MediaGalleryItem[];

    constructor(items: MediaGalleryItem[]) {
        super("MediaGallery");

        this.items = items;
    }
}

export class File extends BaseComponent implements IFile {
    kind: "File" = "File";

    file: UnfurledMediaItem;
    spoiler?: boolean;

    constructor(file: UnfurledMediaItem) {
        super("File");

        this.file = file;
    }

    setSpoiler(spoiler: boolean) {
        this.spoiler = spoiler;
        return this;
    }
}

export class Separator extends BaseComponent implements ISeparator {
    kind: "Separator" = "Separator";
    
    divider?: boolean;
    spacing?: SeparatorSpacingSize;

    constructor() {
        super("Separator");
    }

    setDivider(divider: boolean) {
        this.divider = divider;
        return this;
    }

    setSpacing(spacing: SeparatorSpacingSize) {
        this.spacing = spacing;
        return this;
    }
}

export class Container extends BaseComponent implements IContainer {
    kind: "Container" = "Container";

    accentColor?: number;
    spoiler?: boolean;
    components: AnyComponent[];

    constructor(components: AnyComponent[]) {
        super("Container");

        this.components = components;
    }

    setAccentColor(accentColor: number) {
        this.accentColor = accentColor;
        return this;
    }

    setSpoiler(spoiler: boolean) {
        this.spoiler = spoiler;
        return this;
    }
}

export class Label extends BaseComponent implements ILabel {
    kind: "Label" = "Label";
    
    label: string;
    description?: string;
    component: AnyComponent;

    constructor(component: TextInput);
    constructor(label: string, component: AnyComponent);
    constructor(label: string | TextInput, component?: AnyComponent) {
        super("Label");

        if (label instanceof TextInput) {
            this.label = label.label ?? "";
            this.component = label;
        } else {
            this.label = label;
            this.component = component!;
        }
    }

    setDescription(description: string) {
        this.description = description;
        return this;
    }
}

export class FileUpload extends BaseComponent implements IFileUpload {
    kind: "FileUpload" = "FileUpload";

    customId: string;
    maxValues?: number;
    minValues?: number;
    required?: boolean;

    constructor(name: string, data?: any) {
        super("FileUpload");

        this.customId = encodeInteractionCustomId(name, data ?? null);
    }

    setMinValues(minValues: number) {
        this.minValues = minValues;
        return this;
    }

    setMaxValues(maxValues: number) {
        this.maxValues = maxValues;
        return this;
    }

    setRequired(required: boolean) {
        this.required = required;
        return this;
    }
}

export class Checkbox extends BaseComponent implements ICheckbox {
    kind: "Checkbox" = "Checkbox";

    customId: string;
    default?: boolean;

    constructor(name: string, data?: any) {
        super("Checkbox");

        this.customId = encodeInteractionCustomId(name, data ?? null);
    }
}

export class CheckboxGroup extends BaseComponent implements ICheckboxGroup {
    kind: "CheckboxGroup" = "CheckboxGroup";

    customId: string;
    maxValues?: number;
    minValues?: number;
    required?: boolean;
    options: CheckboxGroupOption[];

    constructor(name: string, options: CheckboxGroupOption[], data?: any) {
        super("CheckboxGroup");

        this.customId = encodeInteractionCustomId(name, data ?? null);
        this.options = options;
    }

    setMinValues(minValues: number) {
        this.minValues = minValues;
        return this;
    }

    setMaxValues(maxValues: number) {
        this.maxValues = maxValues;
        return this;
    }

    setRequired(required: boolean) {
        this.required = required;
        return this;
    }
}

export class CheckboxGroupOption implements ICheckboxGroupOption {
    default?: boolean;
    description?: string;
    label: string;
    value: string;
    
    constructor(label: string, value: string) {
        this.label = label;
        this.value = value;
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
