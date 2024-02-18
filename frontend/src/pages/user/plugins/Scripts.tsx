import { Box, Button, List, ListItem, ListItemButton, ListItemIcon, ListItemText, Paper, Stack, TextField, Typography } from "@mui/material";
import { isErrorResponse, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../../components/FetchData";
import { useSession } from "../../../modules/session/useSession";
import { PluginIcon } from "../../../components/PluginIcon";
import { Link } from "react-router-dom";

let scriptsContext = createFetchDataContext<ScriptPlugin[]>();

export function UserScriptsPage() {
    let session = useSession();

    async function fetchScripts() {
        let scripts = await session.apiClient.getCurrentUserPlugins();
        return scripts;
    }

    return <Box p={1}>
        <Typography variant="h4">New Script Plugin</Typography>
        <Paper sx={{ p: 1, mb: 2 }}>
            <NewPluginForm />
        </Paper>

        <Typography variant="h4">Existing script plugins</Typography>
        <FetchDataGuarded loader={fetchScripts} context={scriptsContext}>
            <ListScripts />
        </FetchDataGuarded>
    </Box>
}

function ListScripts() {
    let { value: scripts } = useFetchedDataBehindGuard(scriptsContext);

    return <Paper>
        <List>
            {scripts.map((v) => <ScriptItem script={v} key={v.id} />)}
            {scripts.length === 0 ? <p>...</p> : null}
        </List>
    </Paper>

}


function ScriptItem({ script }: {
    script: ScriptPlugin,
}) {
    const manageUrl = `/user/plugins/${script.id}/`

    return <ListItem disablePadding>
        <ListItemButton component={Link} to={manageUrl} >
            <ListItemIcon>
                <PluginIcon size="sm" plugin={script} />
            </ListItemIcon>
            <ListItemText primary={script.name} secondary={script.short_description} />
        </ListItemButton>
    </ListItem >
}

function NewPluginForm() {
    const session = useSession();
    const navigate = useNavigate();

    const [name, setName] = useState("");
    const [shortDescription, setShortDescription] = useState("");
    const [longDescription, setLongDescription] = useState("");

    const [isCreating, setIsCreating] = useState(false);

    const [errors, setErrors] = useState<{ name?: string, short?: string, long?: string, general?: string }>({})

    async function create() {
        setIsCreating(true);
        setErrors({});

        let result = await session.apiClient.createPlugin({
            name: name,
            short_description: shortDescription,
            long_description: longDescription,
        })

        if (isErrorResponse(result)) {
            setErrors({
                general: result.response?.description,
                name: result.getFieldError("name"),
                short: result.getFieldError("short_description"),
                long: result.getFieldError("long_description"),
            })
            setIsCreating(false);
        } else {
            navigate(`/user/plugins/${result.id}`)
        }
    }

    return <Stack direction={"column"} spacing={2}>
        <TextField label="Name" variant="outlined"
            error={Boolean(errors.name)} helperText={errors.name}
            onChange={(evt) => setName(evt.target.value)} value={name} />
        <TextField label="Short description" variant="outlined"
            error={Boolean(errors.short)} helperText={errors.short}
            onChange={(evt) => setShortDescription(evt.target.value)} value={shortDescription} />
        <TextField label="Long description" multiline variant="outlined"
            error={Boolean(errors.long)} helperText={errors.long}
            onChange={(evt) => setLongDescription(evt.target.value)} value={longDescription} />

        <Typography variant="body1" color={"error"}>{errors.general}</Typography>
        <Button disabled={isCreating} color="success" onClick={() => create()}>Create!</Button>
    </Stack>
}