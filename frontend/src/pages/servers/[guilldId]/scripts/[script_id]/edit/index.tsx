import { EditGuildScript } from "../../../GuildPage";
import { OurRouteObject } from "../../../../../../misc/ourRoute";

export const routes: OurRouteObject[] = [{
    index: true,
    handle: {
        breadCrumb: (params, data) => {
            if (params.scriptId && data.currentGuildScripts?.scripts) {
                const script = data.currentGuildScripts.scripts.find(v => v.id + "" === params.scriptId)
                if (script) {
                    return script.name + ".ts"
                }
            }

            return params.scriptId ?? "unknown script"
        },
    },
    element: <EditGuildScript />
}]