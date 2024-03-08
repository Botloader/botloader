import { Box, Button, Chip, Divider, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { BotGuild, ErrorCode, isErrorResponse, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { BlLink } from "../../../../components/BLLink";
import { DevConsole } from "../../../../components/DevConsole";
import { useFetchedDataBehindGuard } from "../../../../components/FetchData";
import { GuildSelectionDialog } from "../../../../components/GuildSelectionDialog";
import { useGuilds } from "../../../../modules/guilds/GuildsProvider";
import { pluginContext } from "../../../../components/PluginProvider";
import { ScriptingIde } from "../../../../components/ScriptIde";
import { debugMessageStore } from "../../../../misc/DebugMessages";
import { useSession } from "../../../../modules/session/useSession";

export function EditPluginScriptPage({ initialDiff }: { initialDiff: boolean }) {
    const session = useSession();
    const { value: plugin, setData } = useFetchedDataBehindGuard(pluginContext);
    const [diffSource, setDiffSource] = useState<"published" | "dev" | null>(initialDiff ? "published" : null)
    const cast = plugin as ScriptPlugin;
    const [selectedTestGuild, setSelectedTestGuild] = useState<{
        guildId: string,
        guildName: string,
        scriptId: number,
    } | null>(null);
    const [testGuildSelectionOpen, setTestGuildSelectionOpen] = useState(false);
    const guilds = useGuilds();

    async function selectTestGuild(selection: BotGuild | null) {
        setTestGuildSelectionOpen(false);

        if (!selection) {
            setSelectedTestGuild(null);
            return
        }

        debugMessageStore.pushMessage({
            level: "Client",
            message: "Setting up testing session on " + selection.guild.name,
        })

        const resp = await session.apiClient.addPluginToGuild(plugin.id, selection.guild.id, { auto_update: true });
        if (isErrorResponse(resp)) {
            if (resp.response?.code === ErrorCode.GuildAlreadyHasPlugin) {
                debugMessageStore.pushMessage({
                    level: "Client",
                    message: "Plugin was already added to server",
                })
            } else {
                debugMessageStore.pushMessage({
                    level: "Client",
                    message: "Failed adding plugin to server: " + resp.response?.description,
                })
                return;
            }
        } else {
            debugMessageStore.pushMessage({
                level: "Client",
                message: "Plugin added to server",
            })
        }

        const scriptsResp = await session.apiClient.getAllScripts(selection.guild.id);
        if (isErrorResponse(scriptsResp)) {
            debugMessageStore.pushMessage({
                level: "Client",
                message: "Failed retrieving guild scripts: " + scriptsResp.response?.description,
            })
            return;
        }

        const script = scriptsResp.find((v) => v.plugin_id === plugin.id);
        if (!script) {
            debugMessageStore.pushMessage({
                level: "Client",
                message: "Could not find script? aborting test session",
            })
            return;
        }

        setSelectedTestGuild({
            guildId: selection.guild.id,
            guildName: selection.guild.name,
            scriptId: script.id,
        })

        debugMessageStore.pushMessage({
            level: "Client",
            message: "Started testing session!",
        })

        updateGuildTesting(cast.data.dev_version || "", selection.guild.id, script.id);
    }

    async function updateGuildTesting(content: string, guildId: string, scriptId: number) {

        let resp = await session.apiClient.updateScript(guildId, scriptId, {
            original_source: content,
        });
        if (isErrorResponse(resp)) {
            debugMessageStore.pushMessage({
                level: "Client",
                message: "Failed updating guild script in testing session: " + resp.response?.description,
            });
            return;
        }

        debugMessageStore.pushMessage({
            level: "Client",
            message: "Changes are live are live on testing guild!"
        });
    }

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

            if (selectedTestGuild) {
                await updateGuildTesting(content, selectedTestGuild.guildId, selectedTestGuild.scriptId)
            }
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

            {selectedTestGuild ?
                <Typography variant="body1">Testing session active on: {selectedTestGuild.guildName}</Typography>
                : <Typography mt={1} variant="body1">Test this development version on a server before publishing it</Typography>
            }
            <Button disabled={!Boolean(guilds)} onClick={() => setTestGuildSelectionOpen(true)}>Set testing server</Button>
        </Box>
        <Box sx={{ overflowY: "auto" }}>
            <DevConsole guildId={selectedTestGuild?.guildId ?? undefined} />
        </Box>
        <GuildSelectionDialog
            open={testGuildSelectionOpen}
            onClose={selectTestGuild}
            guilds={guilds?.value?.hasAdmin.filter((v) => v.connected) ?? []}
        />
    </ScriptingIde >
}