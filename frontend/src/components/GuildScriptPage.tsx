import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import {
    Accordion,
    AccordionDetails,
    AccordionSummary,
    Alert,
    Box,
    Button,
    Checkbox,
    Divider,
    FormControl,
    FormControlLabel,
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
    useMemo,
    useRef,
    useState
} from "react";
import { useParams } from "react-router-dom";
import { useCurrentGuildId } from "../modules/guilds/CurrentGuild";
import { FullGuildProvider } from "../modules/guilds/FullGuildProvider";
import { useCurrentGuildScripts } from "../modules/guilds/GuildScriptsProvider";
import { Loading } from "./Loading";
import { PluginIcon } from "./PluginIcon";
import { SelectChannel, SelectCustom, SelectRole } from "./Select";
import ReplayIcon from '@mui/icons-material/Replay';
import { CodeBlock } from './CodeBlock';
import { useSession } from '../modules/session/useSession';
import { ScriptEnableToggle } from './ScriptEnableToggle';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { UseNotifications } from './Notifications';
import { AsyncOpButton } from './AsyncOpButton';
import { BlLink } from './BLLink';

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
        <Paper sx={{ padding: 2 }}>
            {plugin ? <PluginHeader plugin={plugin} script={script} /> : <ScriptHeader script={script} />}
            <Divider sx={{ marginTop: 1 }} />
            <ScriptDetails guildId={currentGuildId} script={script} plugin={plugin} />
        </Paper>
        <FullGuildProvider guildId={currentGuildId}>
            {(script.settings_definitions && script.settings_definitions.length > 0)
                ? <ScriptSettings key={script.id} script={script} />
                : <Paper sx={{ padding: 2, marginTop: 2 }}>
                    <Typography >This script has no settings registered.</Typography>
                </Paper>
            }
        </FullGuildProvider>
    </>
}

function ScriptHeader({ script }: { script: Script }) {
    return <Box>
        <Stack direction={"row"} alignItems={"center"} spacing={1}>
            <Typography variant="h4">{script.name}</Typography>
            <ScriptEnableToggle script={script} />
        </Stack>
    </Box>
}

function PluginHeader({ plugin, script }: { plugin: Plugin, script: Script }) {
    return <Box>
        <Stack direction={"row"} alignItems={"center"} spacing={1}>
            <PluginIcon plugin={plugin} />
            <Typography variant="h4">{plugin.name}</Typography>
            <ScriptEnableToggle script={script} />
        </Stack>
        <Typography mt={1}>{plugin.short_description}</Typography>
    </Box>
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

function stripUnknownFields(script: Script): SettingsOptionValue[] {
    const clone = []
    for (const optionValue of script.settings_values ?? []) {
        const definition = script.settings_definitions?.find(v => v.data.name === optionValue.name)
        if (!definition) {
            console.log(`Could not find option definition for ${optionValue.name}`)
            continue
        }

        if (definition.kind === "List") {
            if (!Array.isArray(optionValue.value)) {
                console.log(`Skipped bad type for list ${optionValue.name}`)
                continue;
            }

            const newRows = []

            for (const row of optionValue.value as SettingsOptionValue[][]) {
                const newRow = []
                for (const field of row) {
                    let fieldDefinition = definition.data.template.find(v => v.name === field.name)
                    if (!fieldDefinition) {
                        console.log(`Skipped unknown value in list ${optionValue.name}, field: ${field.name}`)
                        continue
                    }
                    newRow.push(field)
                }
                newRows.push(newRow)
            }

            clone.push({
                name: optionValue.name,
                value: newRows
            })
        } else {
            clone.push(optionValue)
        }
    }

    return clone
}

function ScriptSettings({ script }: { script: Script }) {
    const session = useSession()
    const currentGuildId = useCurrentGuildId()
    const [newValues, setNewValues] = useState<SettingsOptionValue[]>(() => {
        return stripUnknownFields(script)
    })

    const [isSubmitting, setIsSubmitting] = useState(false)
    const [isValidating, setIsValidating] = useState(false)
    const [apiErrorResponse, setApiErrorResponse] = useState<ApiError | null>(null)
    const notifications = UseNotifications()

    const isDirty = useMemo(() => {
        return JSON.stringify(newValues) !== JSON.stringify(script.settings_values)
    }, [newValues, script])

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
        const response = await session.apiClient.updateScript(currentGuildId, script.id, {
            settings_values: newValues
        })

        if (isErrorResponse(response)) {
            notifications.push({
                class: "error",
                message: "Validation errors occurred"
            })

            setApiErrorResponse(response)
        } else {
            notifications.push({
                class: "success",
                message: "Successfully saved!"
            })
        }
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

        if (!isValidating) {
            setIsValidating(true)
            validateScript().finally(() => setIsValidating(false))
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [newValues, currentGuildId, script.id, session.apiClient])

    return <Paper sx={{ padding: 2, marginTop: 2 }}>
        <Typography variant="h5">Settings</Typography>
        <form
            onSubmit={(evt) => {
                evt.preventDefault()
                if (isSubmitting) {
                    return
                }
                setIsSubmitting(true)
                save().finally(() => setIsSubmitting(false))
            }}
        >
            <SettingsValuesContext.Provider value={settingsHook}>
                {script.settings_definitions?.map(v => {
                    if (v.kind === "List") {
                        return <EditSettingsOptionList key={v.data.name} list={v.data} />
                    } else {
                        return <EditSettingsOption key={v.data.name} option={v.data} />
                    }
                })}
            </SettingsValuesContext.Provider>
            <Button
                type="submit"
                disabled={isSubmitting || !isDirty || isValidating}
                size='large'
                variant='contained'
                color='success'
                fullWidth
                sx={{
                    marginTop: 2,
                    marginBottom: 2
                }}>
                {!isDirty
                    ? "No unsaved changes"
                    : isValidating
                        ? "Validating changes..."
                        : isSubmitting
                            ? "Saving..."
                            : "Save changes"}
            </Button>
        </form>
        <Accordion elevation={2}>
            <AccordionSummary
                expandIcon={<ExpandMoreIcon />}
                aria-controls="panel1-content"
                id="panel1-header"
            >
                Debug info
            </AccordionSummary>
            <AccordionDetails>
                <Typography>Definitions</Typography>
                <CodeBlock>
                    <>{JSON.stringify(script.settings_definitions, null, "  ")}</>
                </CodeBlock>
                <Typography>Values</Typography>
                <CodeBlock>
                    <>{JSON.stringify(newValues, null, "  ")}</>
                </CodeBlock>
            </AccordionDetails>
        </Accordion>
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
            <Typography>{list.label}</Typography>
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
        <Typography variant='body2'>{list.description}</Typography>
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
                                row.push({ name, value: newValue })
                            }
                            setValue(copy)
                        },
                        getFieldError(name) {
                            const prefix = `${list.name}.${i}.`
                            return settingsValuesContext.getFieldError(prefix + name)
                        },
                    }

                    return <Paper key={i}
                        sx={{
                            marginTop: 2,
                            display: "flex",
                            flexDirection: "row",
                            gap: 1,
                            flexWrap: "wrap",
                            padding: 1,
                        }}
                        elevation={3}
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
                        <Divider />
                    </Paper>
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
        || option.kind.kind === "roles"
        || option.kind.kind === "customStringSelect"
        || option.kind.kind === "customStringMultiSelect"
        || option.kind.kind === "customNumberSelect"
        || option.kind.kind === "customNumberMultiSelect") {
        return <EditOptionSelect option={option} />

    } else if (option.kind.kind === "boolean") {
        return <EditOptionBooleanField option={option} />
    }
}

function textFieldValue(rawValue: any, option: SettingsOption) {
    const defaultValue = option.defaultValue ?? ""
    if (rawValue === undefined || rawValue === null) {
        return defaultValue
    }

    return rawValue
}

function booleanValue(rawValue: any, option: SettingsOption) {
    const defaultValue = option.defaultValue ?? false
    if (rawValue === undefined || rawValue === null) {
        return booleanRawValue(defaultValue)
    }

    return booleanRawValue(rawValue)
}

const trueBooleanValues = ["true", "yes", "1"]
function booleanRawValue(rawValue: any) {
    if (typeof rawValue === "string") {
        if (trueBooleanValues.includes(rawValue.toLowerCase())) {
            return true
        }
        return false
    }

    if (typeof rawValue === "number") {
        return rawValue !== 0
    }

    if (typeof rawValue === "boolean") {
        return rawValue
    }

    return false
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
            label={option.label}
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


function EditOptionBooleanField({ option }: { option: SettingsOption }) {
    const { value: rawValue, setValue, error } = useSettingsField(option.name)
    const value = booleanValue(rawValue?.value, option)

    if (option.kind.kind !== "boolean") {
        throw new Error("not a number field")
    }

    return <Box mt={3}>
        <Box>
            <FormControlLabel
                sx={{ marginRight: 0 }}
                label={option.label}
                control={<Checkbox checked={value}
                    onChange={(evt) => {
                        setValue(evt.target.checked)
                    }}
                />}
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

        <FormHelperText error={Boolean(error)}>
            <FieldHelpText description={option.description} error={error} />
        </FormHelperText>
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
            label={option.label}
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
        || option.kind.kind === "channels"
        || option.kind.kind === "customStringSelect"
        || option.kind.kind === "customStringMultiSelect"
        || option.kind.kind === "customNumberSelect"
        || option.kind.kind === "customNumberMultiSelect")

    const { value: rawValue, setValue, error } = useSettingsField(option.name)

    const isMultiple = option.kind.kind === "channels"
        || option.kind.kind === "roles"
        || option.kind.kind === "customStringMultiSelect"
        || option.kind.kind === "customNumberMultiSelect"

    return <Box mt={3} minWidth={"150px"} display={"flex"}>
        <FormControl>
            <InputLabel>{option.label}</InputLabel>
            {isMultiple
                ? <EditOptionSelectMultiple option={option} />
                : <EditOptionSelectSingle option={option} />}

            <FormHelperText error={Boolean(error)}>
                <FieldHelpText description={option.description} error={error} />
            </FormHelperText>

        </FormControl>
        {(Boolean(rawValue) && (option.defaultValue || !option.required)) &&
            <IconButton
                color='warning'
                onClick={() => setValue(null)}
                sx={{ alignSelf: "start" }}
            >
                <ReplayIcon />
            </IconButton>}
    </Box>
}

function EditOptionSelectSingle({ option }: { option: SettingsOption }) {
    const { value: rawValue, setValue, error } = useSettingsField(option.name)

    const value = textFieldValue(rawValue?.value, option)

    if (option.kind.kind === "channel") {
        return <SelectChannel
            multiple={false}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue || null)
            }}
            allowEmpty={!option.required}
            types={option.kind.types ?? undefined}
        />
    } else if (option.kind.kind === "role") {
        return <SelectRole
            multiple={false}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue ?? null)
            }}
            allowEmpty={!option.required}
        />
    } else if (option.kind.kind === "customStringSelect") {
        return <SelectCustom
            multiple={false}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue ?? null)
            }}
            allowEmpty={false}
            options={option.kind.options}
        />
    } else if (option.kind.kind === "customNumberSelect") {
        return <SelectCustom
            multiple={false}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue ?? null)
            }}
            allowEmpty={false}
            options={option.kind.options}
        />
    } else {
        return <>TODO unsupported kind {option.kind.kind}</>
    }
}

function EditOptionSelectMultiple({ option }: { option: SettingsOption }) {
    const { value: rawValue, setValue, error } = useSettingsField(option.name)

    const defaultValue = option.defaultValue ?? []
    const value = rawValue?.value
        ? Array.isArray(rawValue?.value)
            ? rawValue.value
            : defaultValue
        : defaultValue

    if (option.kind.kind === "channels") {
        return <SelectChannel
            multiple={true}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue)
            }}
        />
    } else if (option.kind.kind === "roles") {
        return <SelectRole
            multiple={true}
            value={value}
            label={option.name}
            error={error}
            onChange={(newValue) => {
                setValue(newValue)
            }}
        />
    } else if (option.kind.kind === "customStringMultiSelect") {
        return <SelectCustom
            multiple={true}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue ?? null)
            }}
            options={option.kind.options}
        />
    } else if (option.kind.kind === "customNumberMultiSelect") {
        return <SelectCustom
            multiple={true}
            value={value}
            label={option.label}
            error={error}
            onChange={(newValue) => {
                setValue(newValue ?? null)
            }}
            options={option.kind.options}
        />
    } else {
        return <>TODO unsupported kind {option.kind.kind}</>
    }
}

function FieldHelpText({ description, error }: { description: string, error?: string }) {
    return <>
        {error && <>{error}<br /></>}{description}
    </>
}

function ScriptDetails({ script, guildId, plugin }: {
    script: Script,
    plugin?: Plugin,
    guildId: string,
}) {
    const session = useSession();
    const notifications = UseNotifications();
    const { reload, delScript } = useCurrentGuildScripts();

    async function deleteConfirm() {
        if (window.confirm("are you sure you want to delete this script?")) {
            await delScript(script.id);
        }
    }

    async function updatePluginVersion() {
        const resp = await session.apiClient.updateScriptPlugin(guildId, script.id);
        if (isErrorResponse(resp)) {
            notifications.push({ class: "error", message: "failed updating plugin: " + resp.response?.description });
        } else {
            reload();
            notifications.push({ class: "success", message: "updated plugin" });
        }
    }

    return <Box sx={{
        display: "flex",
        alignItems: "start",
        flexDirection: "column",
    }}>
        <Stack direction={"row"} alignItems="center">
            {plugin ? (
                <>
                    <Typography variant="body1">{plugin.current_version === script.plugin_version_number ?
                        "Using the latest version"
                        : script.plugin_version_number === null
                            ? "Using a modified version"
                            : "New version available"}</Typography>
                    <AsyncOpButton disabled={plugin.current_version === script.plugin_version_number} onClick={updatePluginVersion} label="Update version"></AsyncOpButton>
                    <BlLink disabled={plugin.current_version === script.plugin_version_number}
                        to={`/servers/${guildId}/scripts/${script.id}/edit?diffMode=pluginPublished`}>View changes</BlLink>
                </>
            ) : null}
            <BlLink to={`/servers/${guildId}/scripts/${script.id}/edit`}>
                Edit
            </BlLink>
            <AsyncOpButton label="delete" onClick={() => deleteConfirm()}></AsyncOpButton>
        </Stack>
        {!script.enabled && <Alert severity='warning'>
            This script is not enabled
        </Alert>}
    </Box>
}
