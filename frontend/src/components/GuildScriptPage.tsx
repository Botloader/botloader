import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import {
    Box,
    Button,
    FormControl,
    FormHelperText,
    IconButton,
    InputLabel,
    Paper,
    Stack,
    TextField,
    Typography
} from "@mui/material";
import {
    ApiError,
    Plugin,
    Script,
    SettingsOption,
    SettingsOptionList,
    SettingsOptionValue,
    isErrorResponse
} from "botloader-common";
import {
    createContext,
    useContext,
    useEffect,
    useRef,
    useState
} from "react";
import { useParams } from "react-router-dom";
import { useCurrentGuildId } from "../modules/guilds/CurrentGuild";
import { FullGuildProvider } from "../modules/guilds/FullGuildProvider";
import { useCurrentGuildScripts } from "../modules/guilds/GuildScriptsProvider";
import { Loading } from "./Loading";
import { PluginIcon } from "./PluginIcon";
import { SelectChannel, SelectRole } from "./Select";
import ReplayIcon from '@mui/icons-material/Replay';
import { CodeBlock } from './CodeBlock';
import { useSession } from '../modules/session/useSession';

export function GuildScriptPage() {
    let { scriptId: scriptIdString } = useParams();
    const scripts = useCurrentGuildScripts()
    const currentGuildId = useCurrentGuildId()

    const script = scripts.value?.scripts.find(v => v.id + "" === (scriptIdString as any))
    if (!script) {
        return <Loading />
    }

    const plugin = scripts.value?.plugins.find(v => v.id === script.plugin_id)

    return <>
        {plugin ? <PluginHeader plugin={plugin} /> : <>ToDO</>}
        <FullGuildProvider guildId={currentGuildId}>
            <ScriptSettings script={script} />
        </FullGuildProvider>
    </>
}

function PluginHeader({ plugin }: { plugin: Plugin }) {
    return <Paper sx={{ padding: 2 }}>
        <Stack direction={"row"} alignItems={"center"} spacing={1}>
            <PluginIcon plugin={plugin} />
            <Typography variant="h4">{plugin.name}</Typography>
        </Stack>
        <Typography>{plugin.short_description}</Typography>
    </Paper>
}

interface SettingsValuesHook {
    // values: SettingsOptionValue[]
    setFieldValue: (name: string, value: any) => any,
    getFieldValue: (name: string) => SettingsOptionValue | undefined
    getFieldError: (name: string) => string | undefined
}


const SettingsValuesContext = createContext<SettingsValuesHook | null>(null)

// interface SettingsFieldValueHook{

// }
function useSettingsField(field: string) {
    const hook = useContext(SettingsValuesContext)
    if (!hook) {
        throw new Error("no settings context")
    }

    const error = hook.getFieldError(field)
    const value = hook.getFieldValue(field)
    const setter = (value: any) => hook.setFieldValue(field, value)

    return {
        value,
        error,
        setValue: setter,
    }
}

function ScriptSettings({ script }: { script: Script }) {
    const session = useSession()
    const currentGuildId = useCurrentGuildId()
    const [newValues, setNewValues] = useState<SettingsOptionValue[]>(script.settings_values ?? [])

    const [apiErrorResponse, setApiErrorResponse] = useState<ApiError | null>(null)

    const settingsHook: SettingsValuesHook = {
        getFieldValue(name) {
            return newValues.find(v => v.name === name)
        },
        setFieldValue(name, value) {
            setNewValues(current => {
                const copy = [...current]
                const currentIndex = copy.findIndex(v => name === v.name)
                if (currentIndex !== -1) {
                    if (value === null) {
                        copy.splice(currentIndex, 1)
                    } else {
                        copy[currentIndex] = { name, value }
                    }
                } else if (value !== null) {
                    copy.push({ name, value })
                }

                return copy
            })
        },
        getFieldError(name) {
            const prefix = "settings_values."
            if (apiErrorResponse) {
                const error = apiErrorResponse.getFieldError(prefix + name)
                return error
            }
        },
    }

    async function save() {

    }



    useEffect(() => {
        async function validateScript() {
            setApiErrorResponse(null)

            const response = await session.apiClient.validateScript(currentGuildId, script.id, {
                settings_values: newValues
            })

            if (isErrorResponse(response)) {
                setApiErrorResponse(response)
            }
        }

        validateScript()
    }, [newValues, currentGuildId, script.id, session.apiClient])

    return <Paper sx={{ padding: 2, marginTop: 2 }}>
        <Typography variant="h5">Settings</Typography>
        {script.settings_definitions?.length} definitions and {script.settings_values.length} values
        <SettingsValuesContext.Provider value={settingsHook}>
            {script.settings_definitions?.map(v => {
                if (v.kind === "List") {
                    return <EditSettingsOptionList key={v.data.name} list={v.data} />
                } else {
                    return <EditSettingsOption key={v.data.name} option={v.data} />
                }
            })}
        </SettingsValuesContext.Provider>
        <Paper elevation={3} sx={{ p: 2 }}>
            <Typography>Debug definitions</Typography>
            <CodeBlock>
                <>{JSON.stringify(script.settings_definitions, null, "  ")}</>
            </CodeBlock>
            <Typography>Debug</Typography>

            {Object.entries(newValues).map(([k, v]) => <Box>
                {k}:{" "}
                <code key={k}>
                    {JSON.stringify(v, null, " ")}
                </code></Box>)}
        </Paper>
    </Paper >
}

function EditSettingsOptionList({ list }: {
    list: SettingsOptionList,
}) {
    const { value: rawValue, setValue } = useSettingsField(list.name)
    const settingsValuesContext = useContext(SettingsValuesContext)
    if (!settingsValuesContext) {
        throw new Error("no settings context")
    }


    const value = rawValue === undefined
        ? []
        : Array.isArray(rawValue.value)
            ? rawValue.value as SettingsOptionValue[][]
            : []


    function newItem() {

        setValue([
            ...value,
            [],
        ])
    }

    function deleteItem(index: number) {
        const copy = [...value]
        copy.splice(index, 1)
        setValue(copy)
    }

    return <Paper sx={{ marginTop: 3, padding: 1 }} elevation={2}>
        <Box display={"flex"} justifyContent={"space-between"}>
            <Typography>{list.name}</Typography>
            <Box>
                <Button
                    onClick={newItem}
                    variant="contained"
                    size="small"
                    color="success">
                    <AddIcon />
                </Button>
            </Box>
        </Box>
        <Box paddingLeft={1}>
            {
                value.map((item, i) => {

                    const innerContext: SettingsValuesHook = {
                        getFieldValue(name) {
                            return item.find(v => v.name === name)
                        },
                        setFieldValue(name, newValue) {
                            const copy = [...value]
                            const row = copy[i]
                            const currentIndex = row.findIndex(v => name === v.name)
                            console.log("Setting", name, "to", newValue, currentIndex)
                            if (currentIndex !== -1) {
                                if (newValue === null) {
                                    row.splice(currentIndex, 1)
                                } else {
                                    row[currentIndex] = { name, value: newValue }
                                }
                            } else if (value !== null) {
                                console.log("NOT NULL!")
                                row.push({ name, value: newValue })
                            }
                            setValue(copy)
                            console.log(newValue)
                        },
                        getFieldError(name) {
                            const prefix = `${list.name}.${i}.`
                            return settingsValuesContext.getFieldError(prefix + name)
                        },
                    }

                    return <Box key={i}
                        display={"flex"}
                        flexDirection={"row"}
                        gap={1}
                    >
                        <SettingsValuesContext.Provider value={innerContext}>
                            {list.template.map(v => (
                                // <Grid2 key={v.name} xs>
                                <EditSettingsOption key={v.name} option={v} />
                                // </Grid2>
                            ))}
                        </SettingsValuesContext.Provider>
                        <Box alignSelf={"center"}>
                            <Button color="error" onClick={() => {
                                deleteItem(i)
                                console.log("deleting", i)
                            }}><DeleteIcon /></Button>
                        </Box>
                    </Box>
                })
            }
        </Box>
    </Paper>
}

function EditSettingsOption({ option }: { option: SettingsOption, value?: string | number }) {
    if (option.kind.kind === "float"
        || option.kind.kind === "integer"
        || option.kind.kind === "integer64") {
        return <EditOptionNumberField option={option} />

    } else if (option.kind.kind === "string") {
        return <EditOptionTextField option={option} />

    } else if (option.kind.kind === "channel"
        || option.kind.kind === "role"
        || option.kind.kind === "channels"
        || option.kind.kind === "roles") {
        return <EditOptionSelect option={option} />

    }
}

function textFieldValue(rawValue: any, option: SettingsOption) {
    const defaultValue = option.defaultValue ?? ""
    if (rawValue === undefined || rawValue === null) {
        return defaultValue
    }

    return rawValue
}

function EditOptionNumberField({ option }: { option: SettingsOption }) {
    const inputRef = useRef<HTMLInputElement | null>(null)
    const { value: rawValue, setValue, error } = useSettingsField(option.name)
    const value = textFieldValue(rawValue?.value, option)

    if (option.kind.kind !== "float"
        && option.kind.kind !== "integer"
        && option.kind.kind !== "integer64") {
        throw new Error("not a number field")
    }

    const isInteger = option.kind.kind === "integer64"
        || option.kind.kind === "integer"

    useEffect(() => {
        if (!inputRef.current) {
            return
        }

        if ((value?.toString() ?? "") !== inputRef.current.value) {
            inputRef.current.value = value?.toString() ?? ""
        }
    }, [value, inputRef])

    return <Box mt={3}>
        <TextField
            id={"script-option-" + option.name}
            label={option.name}
            // defaultValue="Default Value"
            type={"number"}
            inputProps={{
                step: isInteger ? 1 : undefined,
                min: option.kind.min,
                max: option.kind.max,
            }}
            inputRef={inputRef}
            helperText={<FieldHelpText description={option.description} error={error} />}
            onBlur={(evt) => {
                const newValue = evt.target.value
                if (evt.target.value === "") {
                    setValue(null)
                    evt.target.value = ""
                } else {
                    if (option.kind.kind === "float") {
                        const parsed = parseFloat(newValue)
                        const realNewValue = isNaN(parsed) ? 0 : parsed
                        setValue(realNewValue)
                    } else if (option.kind.kind === "integer") {
                        const parsed = parseFloat(newValue.replaceAll(".", ""))
                        const realNewValue = isNaN(parsed) ? 0 : parsed
                        setValue(realNewValue)
                    } else {
                        setValue(newValue)
                    }


                }
            }}
            error={Boolean(error)}
        />
        {(Boolean(rawValue) && (option.defaultValue || !option.required)) &&
            <IconButton
                color='warning'
                onClick={() => {
                    setValue(null)
                }}
            >
                <ReplayIcon />
            </IconButton>}
    </Box>
}


function EditOptionTextField({ option }: { option: SettingsOption }) {
    if (option.kind.kind !== "string") {
        throw new Error("not a string field")
    }

    const { value: rawValue, setValue, error } = useSettingsField(option.name)
    const value = textFieldValue(rawValue?.value, option)

    return <Box mt={3}>
        <TextField
            id={"script-option-" + option.name}
            label={option.name}
            // defaultValue="Default Value"
            type={"text"}
            inputProps={{
                maxLength: option.kind.max_length,
                minLength: option.kind.min_length,
            }}
            helperText={<FieldHelpText description={option.description} error={error} />}
            onChange={(evt) => {
                setValue(evt.target.value)
            }}
            value={value}
            error={Boolean(error)}
        />
        {(Boolean(rawValue) && (option.defaultValue || !option.required)) &&
            <IconButton
                color='warning'
                onClick={() => setValue(null)}
            >
                <ReplayIcon />
            </IconButton>}
    </Box>
}

function EditOptionSelect({ option }: { option: SettingsOption }) {
    console.assert(option.kind.kind === "channel"
        || option.kind.kind === "role"
        || option.kind.kind === "roles"
        || option.kind.kind === "channels")

    const { error } = useSettingsField(option.name)

    const isMultiple = option.kind.kind === "channels" || option.kind.kind === "roles"

    return <Box mt={3} minWidth={"150px"}>
        <FormControl fullWidth>
            <InputLabel>{option.name}</InputLabel>
            {isMultiple
                ? <EditOptionSelectMultiple option={option} />
                : <EditOptionSelectSingle option={option} />}
            <FormHelperText error={Boolean(error)}>
                <FieldHelpText description={option.description} error={error} />
            </FormHelperText>
        </FormControl>
    </Box>
}

function EditOptionSelectSingle({ option }: { option: SettingsOption }) {
    const { value: rawValue, setValue, error } = useSettingsField(option.name)

    console.assert(option.kind.kind === "channel" || option.kind.kind === "role")

    const value = rawValue?.value
        ? typeof rawValue.value === "string"
            ? rawValue.value
            : ""
        : ""

    if (option.kind.kind === "channel") {
        return <SelectChannel
            multiple={false}
            value={value}
            label={option.name}
            error={error}
            onChange={(newValue) => {
                setValue(newValue)
            }}
        />
    }

    return <SelectRole
        multiple={false}
        value={value}
        label={option.name}
        error={error}
        onChange={(newValue) => {
            setValue(newValue)
        }}
    />
}

function EditOptionSelectMultiple({ option }: { option: SettingsOption }) {
    const { value: rawValue, setValue, error } = useSettingsField(option.name)

    console.assert(option.kind.kind === "channels" || option.kind.kind === "roles")

    const value = rawValue?.value
        ? Array.isArray(rawValue.value)
            ? rawValue.value as string[]
            : []
        : []

    if (option.kind.kind === "channels") {
        return <SelectChannel
            multiple={true}
            value={value}
            label={option.name}
            error={error}
            onChange={(newValue) => {
                setValue(newValue)
            }}
        />
    }

    return <SelectRole
        multiple={true}
        value={value}
        label={option.name}
        error={error}
        onChange={(newValue) => {
            setValue(newValue)
        }}
    />
}

function FieldHelpText({ description, error }: { description: string, error?: string }) {
    return <>
        {error && <>{error}<br /></>}{description}
    </>
}