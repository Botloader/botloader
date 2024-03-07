import { Switch } from "@mui/material";
import { Script } from "botloader-common";
import { useState } from "react";
import { useCurrentGuildScripts } from "../modules/guilds/GuildScriptsProvider";

export function ScriptEnableToggle({ script }: { script: Script }) {
    const [isToggling, setIsToggling] = useState(false)
    const { toggleScript } = useCurrentGuildScripts();

    return <Switch
        checked={script.enabled}
        disabled={isToggling}
        color={"success"}
        onChange={(evt) => {
            setIsToggling(true)
            toggleScript(script.id, evt.target.checked)
                .finally(() => setIsToggling(false))
        }} />
}