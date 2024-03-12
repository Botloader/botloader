import { RouteObject } from "react-router-dom";
import { UserPremiumPage } from "./Premium";

export const routes: RouteObject[] = [
    {
        index: true,
        handle: {
            breadCrumb: () => "Premium"
        },
        element: <UserPremiumPage />
    }
]