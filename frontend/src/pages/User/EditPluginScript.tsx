import { Box, Button, Chip, Divider, Typography } from "@mui/material";
import { ScriptPlugin } from "botloader-common";
import { DevConsole } from "../../components/DevConsole";
import { useFetchedDataBehindGuard } from "../../components/FetchData";
import { pluginContext } from "../../components/PluginProvider";
import { ScriptEditor } from "../../components/ScriptEditor";
import { useSession } from "../../components/Session";
import { debugMessageStore } from "../../misc/DebugMessages";

export function EditPluginScriptPage({ isDiff }: { isDiff: boolean }) {
    const session = useSession();
    const { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    const cast = plugin as ScriptPlugin;

    async function save(content: string) {
        debugMessageStore.pushMessage({
            level: "Client",
            message: "Saving...",
        })
        await session.apiClient.updateScriptPluginDevVersion(plugin.id, { source: content });
        debugMessageStore.pushMessage({
            level: "Client",
            message: "Saved",
        })
    }

    return <Box sx={{ display: "flex", flexDirection: "row", flexGrow: 1 }}>
        <Box sx={{ flexGrow: 1, marginRight: "250px" }}>
            <ScriptEditor
                initialSource={cast.data.dev_version || null}
                onSave={save}
                files={[]}
                isDiffEditor={isDiff}
                originalDiffSource={cast.data.published_version || ""}
            />
        </Box>
        <Box width={250} display="flex" flexDirection="column" position={"absolute"} top={69} bottom={0} right={0}>
            <Box p={1}>
                <Typography>Editing development version of plugin <Chip variant="outlined" label={plugin.name} /></Typography>
                <Button color="primary" href={`/user/plugins/${plugin.id}`}>Back</Button>
                <Divider />
                <Typography mt={1} variant="body1">Test this development version on a server before publishing it</Typography>
                <Button>Test on a server</Button>
            </Box>
            <Box sx={{ overflowY: "auto" }}>
                <DevConsole />
            </Box>
        </Box>
    </Box >
}