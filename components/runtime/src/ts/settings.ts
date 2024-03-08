import { OpWrappers } from "./op_wrappers"
import * as Internal from "./generated/internal/index";
import { ChannelType } from "./discord/index";

type DefaultValue<TDefault> = TDefault extends undefined
    ? { defaultValue?: TDefault }
    : { defaultValue: TDefault }

type BaseOptions<TRequired, TDefault> = {
    label?: string
    description?: string
    required?: TRequired
} & DefaultValue<TDefault>

type StringOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    maxLength?: number,
    minLength?: number,
}

type FloatOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    max?: number,
    min?: number,
}

type IntegerOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    max?: number,
    min?: number,
}

type Integer64Options<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    max?: string,
    min?: string,
}

type ChannelOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    types?: ChannelType[]
}

type ChannelsOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    types?: ChannelType[]
    max?: number,
}

type RoleOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    requireAssignable?: boolean
}

type RolesOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    requireAssignable?: boolean
    max?: number,
}

interface OptionTypes<TRequired, TDefault> {
    string: StringOptions<TRequired, TDefault>,
    float: FloatOptions<TRequired, TDefault>,
    integer: IntegerOptions<TRequired, TDefault>,
    integer64: Integer64Options<TRequired, TDefault>,
    channel: ChannelOptions<TRequired, TDefault>,
    channels: ChannelsOptions<TRequired, TDefault>,
    role: RoleOptions<TRequired, TDefault>,
    roles: RolesOptions<TRequired, TDefault>,
}

type OptionTypesKeys = keyof OptionTypes<boolean, any>

type OptionTypesUnion = {
    [Key in OptionTypesKeys]: { kind: Key } & OptionTypes<boolean, any>[Key]
}[OptionTypesKeys]

export interface ConfigOptionSetValueTypesMapping {
    string: string,
    integer: number,
    integer64: string,
    float: number,
    channel: string,
    channels: string[],
    role: string,
    roles: string[],
}

type InnerToConfigSetValueType<TKind extends OptionTypesKeys> =
    ConfigOptionSetValueTypesMapping[TKind]

// Converts the input generic parameters (kind, default value, required)
// to the value type depending on if the config option is required or has a default value
type ToConfigValue<
    TKind extends OptionTypesKeys,
    TDefaultValue,
    TRequired extends boolean
> =
    TRequired extends true
    ? InnerToConfigSetValueType<TKind>
    : TDefaultValue extends undefined
    ? InnerToConfigSetValueType<TKind> | SysDefaultValueType<TKind>
    : InnerToConfigSetValueType<TKind> | TDefaultValue | SysDefaultValueType<TKind>

const SysDefaultValues = {
    string: "",
    float: undefined,
    integer: undefined,
    integer64: undefined,
    channel: undefined,
    channels: [] as string[],
    role: undefined,
    roles: [] as string[],
}

type SysDefaultValueType<TKind extends keyof typeof SysDefaultValues> =
    (typeof SysDefaultValues)[TKind]

function getSysDefaultValue<TKind extends keyof typeof SysDefaultValues>
    (kind: TKind): SysDefaultValueType<TKind> {
    return SysDefaultValues[kind]
}


type OptionDefinitionOption = {
    kind: "Option",
    name: string,
    definition: OptionTypesUnion,
}

type OptionDefinitionList = {
    kind: "List",
    name: string,
    template: Record<string, OptionTypesUnion>,
    options: ListOptions<any>,
}

type AnyOptionDefinition = OptionDefinitionOption | OptionDefinitionList

type ListOptions<TDefault> = {
    description?: string,
    label?: string,
    max?: number,
    min?: number,
    defaultValue?: TDefault,
}

export class LoadedOption<TKind extends OptionTypesKeys, TDefaultValue, TRequired extends boolean> {
    definition: OptionDefinitionOption
    value: ToConfigValue<TKind, TDefaultValue, TRequired>

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionOption, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (rawValue) {
            this.value = loadOptionValue(rawValue.value, definition.definition)
        } else {
            this.value = definition.definition.defaultValue ?? getSysDefaultValue(definition.definition.kind)
        }
    }
}

type TDefaultValueTopLevel<TRequired extends boolean, TKind extends OptionTypesKeys> =
    TRequired extends true
    ? InnerToConfigSetValueType<TKind>
    : InnerToConfigSetValueType<TKind> | undefined;

export class SettingsManager {

    definitions: AnyOptionDefinition[] = [];
    loadedSettings: Internal.SettingsOptionValue[];

    constructor(scriptId: number) {
        this.loadedSettings = OpWrappers.getSettings(scriptId)
    }

    addOptionString<
        TDefault extends TDefaultValueTopLevel<TRequired, "string">,
        TRequired extends boolean = false,
    >(name: string, options: StringOptions<TRequired, TDefault>) {
        return this.addOption<"string", TRequired, TDefault>(name, "string", options)
    }

    addOptionFloat<
        TDefault extends TDefaultValueTopLevel<TRequired, "float">,
        TRequired extends boolean = false,
    >(name: string, options: FloatOptions<TRequired, TDefault>) {
        return this.addOption<"float", TRequired, TDefault>(name, "float", options)
    }

    addOptionInteger<
        TDefault extends TDefaultValueTopLevel<TRequired, "integer">,
        TRequired extends boolean = false,
    >(name: string, options: IntegerOptions<TRequired, TDefault>) {
        return this.addOption<"integer", TRequired, TDefault>(name, "integer", options)
    }

    addOptionInteger64<
        TDefault extends TDefaultValueTopLevel<TRequired, "string">,
        TRequired extends boolean = false,
    >(name: string, options: Integer64Options<false, TDefault>) {
        return this.addOption<"integer64", false, TDefault>(name, "integer64", options)
    }

    addOptionRole<
        TDefault extends string | undefined = undefined,
    >(name: string, options: RoleOptions<false, TDefault>) {
        return this.addOption<"role", false, TDefault>(name, "role", options)
    }

    addOptionRoles<
        TDefault extends string[] | undefined = undefined,
    >(name: string, options: RolesOptions<false, TDefault>) {
        return this.addOption<"roles", false, TDefault>(name, "roles", options)
    }

    addOptionChannel<
        TDefault extends string | undefined = undefined,
    >(name: string, options: ChannelOptions<false, TDefault>) {
        return this.addOption<"channel", false, TDefault>(name, "channel", options)
    }

    addOptionChannels<
        TDefault extends string[] | undefined = undefined,
    >(name: string, options: ChannelsOptions<false, TDefault>) {
        return this.addOption<"channels", false, TDefault>(name, "channels", options)
    }

    private addOption<
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigSetValueType<TKind> | undefined = undefined,
    >(name: string, kind: TKind, options: OptionTypes<TRequired, TDefault>[TKind]): LoadedOption<TKind, TDefault, TRequired> {
        const definition: OptionDefinitionOption = {
            kind: "Option",
            name,
            definition: { kind, ...options } as any,
        }

        this.definitions.push(definition)

        const value = this.loadedSettings.find(v => v.name === name)
        return new LoadedOption(definition, value)
    }

    startList(name: string) {
        return new ListBuilder<{}>(name, this)
    }

    /**
     * @internal
     */
    toInternalOptions(): Internal.SettingsOptionDefinition[] {
        return this.definitions.map(def => defToInternal(def))
    }
}

interface OptionsMap {
    [key: string]: { defaultValue: any, kind: any, required: boolean }
}

type LayerOption<TInner, TKey extends string, TKind extends keyof ConfigOptionSetValueTypesMapping, TDefault, TRequired> =
    { [Prop in keyof TInner]: TInner[Prop] } & { [Prop in TKey]: { defaultValue: TDefault, kind: TKind, required: TRequired } };

type OptionMapValues<T extends OptionsMap> = {
    [Prop in keyof T]: ToConfigValue<
        T[Prop]["kind"],
        T[Prop]["defaultValue"],
        T[Prop]["required"]
    >
}

// interface LoadedList<TOpts extends OptionsMap> {
//     definitions: AnyOptionDefinition[],
//     value: OptionMapValues<TOpts>[]
// }

export class LoadedList<TOpts extends OptionsMap> {
    definition: OptionDefinitionList
    value: OptionMapValues<TOpts>[]

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionList, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (!rawValue?.value) {
            this.value = definition.options.defaultValue ?? []
            return
        }

        const arrValue = rawValue?.value
        if (!Array.isArray(arrValue)) {
            this.value = definition.options.defaultValue ?? []
            return
        }

        this.value = []

        for (const entry of arrValue) {
            if (!Array.isArray(entry)) {
                console.log(`Invalid list item in option list ${definition.name} skipping...`)
                continue
            }

            const obj = {} as any
            for (const [name, optionDefinition] of Object.entries(definition.template)) {
                const optionValue = entry.find(v => "name" in v && v.name === name)
                if (optionValue && "value" in optionValue) {
                    obj[name] = loadOptionValue(optionValue.value, optionDefinition)
                } else {
                    if (optionDefinition.required && optionDefinition.defaultValue === undefined) {
                        console.log(`List item in option list ${definition.name} missing required option [${name}], skipping...`)
                    } else {
                        obj[name] = optionDefinition.defaultValue ?? getSysDefaultValue(optionDefinition.kind)
                    }
                }
            }

            this.value.push(obj)
        }
    }
}

export class ListBuilder<TOpts extends OptionsMap> {
    manager: SettingsManager;
    name: string;
    definitions: Record<string, OptionTypesUnion> = {};

    constructor(name: string, manager: SettingsManager) {
        this.name = name
        this.manager = manager
    }

    addOptionString<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: StringOptions<TRequired, TDefault>) {
        return this.addOption<TName, "string", TRequired, TDefault>(name, "string", options)
    }

    addOptionFloat<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number | undefined = undefined,
    >(name: TName, options: FloatOptions<TRequired, TDefault>) {
        return this.addOption<TName, "float", TRequired, TDefault>(name, "float", options)
    }

    addOptionInteger<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number | undefined = undefined,
    >(name: TName, options: IntegerOptions<TRequired, TDefault>) {
        return this.addOption<TName, "integer", TRequired, TDefault>(name, "integer", options)
    }

    addOptionInteger64<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: Integer64Options<TRequired, TDefault>) {
        return this.addOption<TName, "integer64", TRequired, TDefault>(name, "integer64", options)
    }

    addOptionRole<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: RoleOptions<TRequired, TDefault>) {
        return this.addOption<TName, "role", TRequired, TDefault>(name, "role", options)
    }

    addOptionRoles<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string[] | undefined = undefined,
    >(name: TName, options: RolesOptions<TRequired, TDefault>) {
        return this.addOption<TName, "roles", TRequired, TDefault>(name, "roles", options)
    }

    addOptionChannel<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: ChannelOptions<TRequired, TDefault>) {
        return this.addOption<TName, "channel", TRequired, TDefault>(name, "channel", options)
    }

    addOptionChannels<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string[] | undefined = undefined,
    >(name: TName, options: ChannelsOptions<TRequired, TDefault>) {
        return this.addOption<TName, "channels", TRequired, TDefault>(name, "channels", options)
    }

    private addOption<
        const TName extends string,
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigSetValueType<TKind> | undefined = undefined,
    >(name: TName, kind: TKind, options: OptionTypes<TRequired, TDefault>[TKind]) {
        this.definitions[name] = {
            kind: kind,
            ...(options as any),
        }

        return this as ListBuilder<LayerOption<TOpts, TName, TKind, TDefault, TRequired>>
    }

    complete(options?: ListOptions<OptionMapValues<TOpts>[]>): LoadedList<TOpts> {

        const def: OptionDefinitionList = {
            kind: "List",
            name: this.name,
            options: options ?? {},
            template: this.definitions,
        }

        this.manager.definitions.push(def)

        const value = this.manager.loadedSettings.find(v => v.name === this.name)
        return new LoadedList(def, value)
    }
}

function defToInternal(def: AnyOptionDefinition): Internal.SettingsOptionDefinition {
    if (def.kind === "Option") {
        return {
            kind: "Option",
            data: {
                name: def.name,
                label: def.definition.label ?? def.name,
                description: def.definition.description || "",
                required: def.definition.required ?? false,
                kind: optionTypesUnionToInternal(def.definition),
                defaultValue: def.definition.defaultValue ?? null,
            }
        }
    } else {
        return {
            kind: "List",
            data: {
                name: def.name,
                description: def.options.description || "",
                label: def.options.label ?? def.name,
                // asd: 10,
                required: false,
                defaultValue: def.options.defaultValue ?? null,
                template: Object.entries(def.template).map(([k, v]) => ({
                    name: k,
                    label: v.label ?? k,
                    defaultValue: v.defaultValue ?? null,
                    description: v.description ?? "",
                    required: v.required ?? false,
                    kind: optionTypesUnionToInternal(v),
                }))
            }
        }
    }
}

function optionTypesUnionToInternal(def: OptionTypesUnion): Internal.SettingsOptionType {
    switch (def.kind) {
        case "string":
            return {
                kind: "string",
                max_length: def.maxLength ?? null,
                min_length: def.minLength ?? null,
            }
        case "float":
            return {
                kind: "float",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "integer":
            return {
                kind: "integer",
                max: null,
                min: null,
            }
        case "integer64":
            return {
                kind: "integer64",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "channel":
            return {
                kind: "channel",
                types: def.types ?? null,
            }
        case "channels":
            return {
                kind: "channels",
                types: def.types ?? null,
                max_length: def.max ?? null,
                min_length: 0
            }
        case "role":
            return {
                kind: "role",
                assignable: def.requireAssignable ?? null,
            }
        case "roles":
            return {
                kind: "roles",
                assignable: def.requireAssignable ?? null,
                max_length: def.max ?? null,
                min_length: 0
            }
    }
}

// I originally tried to use generics here to double check the return type 
// but i seem to run into weird typescript limitations regaring that
// and it also wouldn't be usable from the list then'
//
// This performs some basic protections and sanity checks, but more in depth ones may be implemented later
function loadOptionValue(value: any, definition: OptionTypesUnion): any {
    switch (definition.kind) {
        case "string": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "float": {
            return typeof value === "number" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "integer": {
            return typeof value === "number" ? Math.floor(value) : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "integer64": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "channel": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "channels": {
            return Array.isArray(value) ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "role": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "roles": {
            return Array.isArray(value) ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
    }
}
