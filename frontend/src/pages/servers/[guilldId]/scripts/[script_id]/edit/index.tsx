import { EditGuildScript } from "../../../GuildPage";
import { OurRouteObject } from "../../../../../../misc/ourRoute";

export const routes: OurRouteObject[] = [{
    index: true,
    handle: {
        breadCrumb: (params, data) => {
            return "Edit"
        },
    },
    element: <EditGuildScript />
}]