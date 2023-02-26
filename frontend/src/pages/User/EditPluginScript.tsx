import { Box, Button, Chip, Divider, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { isErrorResponse, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { BlLink } from "../../components/BLLink";
import { DevConsole } from "../../components/DevConsole";
import { useFetchedDataBehindGuard } from "../../components/FetchData";
import { pluginContext } from "../../components/PluginProvider";
import { ScriptingIde } from "../../components/ScriptIde";
import { useSession } from "../../components/Session";
import { debugMessageStore } from "../../misc/DebugMessages";

export function EditPluginScriptPage({ initialDiff }: { initialDiff: boolean }) {
    const session = useSession();
    const { value: plugin, setData } = useFetchedDataBehindGuard(pluginContext);
    const [diffSource, setDiffSource] = useState<"published" | "dev" | null>(initialDiff ? "published" : null)
    const cast = plugin as ScriptPlugin;

    async function save(content: string) {
        debugMessageStore.pushMessage({
            level: "Client",
            message: "Saving...",
        })
        const resp = await session.apiClient.updateScriptPluginDevVersion(plugin.id, { source: content });
        if (!isErrorResponse(resp)) {
            setData(resp);
            debugMessageStore.pushMessage({
                level: "Client",
                message: "Saved",
            })
        }
    }

    return <ScriptingIde
        initialSource={cast.data.dev_version || undefined}
        onSave={save}
        files={[]}
        isDiffEditor={diffSource !== null}
        diffSource={diffSource === "dev" ? cast.data.dev_version ?? "" : cast.data.published_version ?? ""}
    >
        <Box p={1}>
            <Typography>Editing development version of plugin</Typography>
            <Chip variant="outlined" label={plugin.name} />
            <BlLink to={`/user/plugins/${plugin.id}`}>Back</BlLink>
            <Divider sx={{ mb: 1 }} />
            <Typography>Diff mode</Typography>
            <ToggleButtonGroup
                color="primary"
                value={diffSource === null ? "off" : diffSource}
                exclusive
                onChange={(_, value) => {
                    if (value === "off") {
                        setDiffSource(null)
                    } else {
                        setDiffSource(value as "published" | "dev");
                    }
                }}
                aria-label="Diff mode"
            >
                <ToggleButton value="published">Published</ToggleButton>
                <ToggleButton value="dev">Dev</ToggleButton>
                <ToggleButton value="off">Off</ToggleButton>
            </ToggleButtonGroup>
            <Divider sx={{ mb: 1 }} />
            <Typography mt={1} variant="body1">Test this development version on a server before publishing it</Typography>
            <Button>Test on a server</Button>
        </Box>
        <Box sx={{ overflowY: "auto" }}>
            <DevConsole />
        </Box>
    </ScriptingIde >
}