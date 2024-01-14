import { RouteObject } from "react-router-dom";
import { UserGeneralPage } from "./General";

export const routes: RouteObject[] = [
    {
        index: true,
        element: <UserGeneralPage />
    }
]