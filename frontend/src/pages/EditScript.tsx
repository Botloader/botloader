import { BotGuild, isErrorResponse, Script } from "botloader-common";
import { useRef, useState } from "react";
import { useSession } from "../components/Session";
import "./EditScript.css";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { debugMessageStore } from "../misc/DebugMessages";
import { DevConsole } from "../components/DevConsole";
import { useBotloaderMonaco } from "../components/BotloaderSdk";
import { createFetchDataContext, FetchData, useFetchedDataBehindGuard } from "../components/FetchData";
import { ScriptEditor } from "../components/ScriptEditor";
import { Button } from "@mui/material";

export const scriptsContext = createFetchDataContext<Script[]>();

export function EditScriptPage(props: { guild: BotGuild, scriptId: number }) {
    const session = useSession();

    async function fetchScripts() {
        let resp = await session.apiClient.getAllScripts(props.guild.guild.id);
        return resp;
    }

    return <FetchData loader={fetchScripts} context={scriptsContext}>
        <InnerPage guild={props.guild} scriptId={props.scriptId} />
    </FetchData>
}

export function InnerPage(props: { guild: BotGuild, scriptId: number }) {
    const { value: scripts } = useFetchedDataBehindGuard(scriptsContext);
    const script = scripts.find((v) => v.id === props.scriptId);
    const loaded = useBotloaderMonaco(scripts
        .filter((v) => v.id !== props.scriptId)
        .map((v) => ({ name: v.name, content: v.original_source })))

    if (!script) {
        return <>Unknown script</>
    } else if (loaded) {
        return <LoadedNew guild={props.guild} script={script}></LoadedNew>
    } else {
        return <>Loading...</>
    }
}

function LoadedNew(props: { guild: BotGuild, script: Script }) {
    const { value: scripts, setData } = useFetchedDataBehindGuard(scriptsContext);
    const session = useSession();
    const [isDirty, setIsDirty] = useState(false);
    const newSource = useRef(props.script.original_source);

    async function setScript(newScript: Script) {
        setData((current) => {
            let cop = [...(current ?? [])];
            let index = cop.findIndex((v) => v.id === newScript.id);
            if (index >= 0) {
                cop[index] = newScript;
            }

            return cop;
        })
    }

    async function toggleScript(scriptId: number, enabled: boolean) {
        let newScript = await session.apiClient.updateScript(props.guild.guild.id, scriptId, {
            enabled,
        });
        if (!isErrorResponse(newScript)) {
            setScript(newScript);
        }
        await session.apiClient.reloadGuildVm(props.guild.guild.id);
    }

    async function save(content: string) {
        debugMessageStore.pushMessage({
            guildId: props.guild.guild.id,
            level: "Client",
            message: "Saving..."
        });

        console.log("Saving!");

        let updated = await session.apiClient.updateScript(props.guild.guild.id, props.script.id, {
            original_source: content,
        });
        if (!isErrorResponse(updated)) {
            setScript(updated);
        }

        await session.apiClient.reloadGuildVm(props.guild.guild.id);
        debugMessageStore.pushMessage({
            guildId: props.guild.guild.id,
            level: "Client",
            message: "Changes are live!"
        });

        setIsDirty(false);
    }

    return <div className="scripting-ide">
        <ScriptEditor
            initialSource={props.script.original_source}
            onSave={save}
            files={scripts.filter((v) => v.id !== props.script.id)
                .map((v) => ({ name: v.name, content: v.original_source }))}
            onChange={(source) => {
                newSource.current = source ?? "";
                if (source !== props.script.original_source) {
                    if (!isDirty) {
                        setIsDirty(true);
                    }
                } else {
                    if (isDirty) {
                        setIsDirty(false);
                    }
                }
            }}
        />
        <div className="scripting-panel">
            <div className="scripting-actions">
                <h3 className="scripting-header">Editing {props.script.name}.ts</h3>
                <div className="actions-row">
                    <Button href={`/servers/${props.guild.guild.id}`}>Back to server page</Button>
                </div>
                <div className="actions-row">
                    <p>Script is {props.script.enabled ? <span className="status-good">Enabled</span> : <span className="status-bad">Disabled</span>}</p>
                    {props.script.enabled ?
                        <AsyncOpButton className="primary" label="Disable" onClick={() => toggleScript(props.script.id, false)}></AsyncOpButton>
                        :
                        <AsyncOpButton className="primary" label="Enable" onClick={() => toggleScript(props.script.id, true)}></AsyncOpButton>
                    }
                </div>
                <div className="actions-row">
                    {isDirty ?
                        <AsyncOpButton className="primary" label="Save" onClick={() => save(newSource.current)}></AsyncOpButton>
                        : <p>No changes made</p>}
                </div>
            </div>
            <div className="scripting-console">
                <DevConsole guildId={props.guild.guild.id} />
            </div>
        </div>
    </div>
}
