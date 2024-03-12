import { Box, Typography } from "@mui/material";
import { RouteObject } from "react-router-dom";
import { BlLink } from "../../components/BLLink";

export const routes: RouteObject[] = [
    {
        index: true,
        element: <ConfirmPurchasePage />
    }
]


export function ConfirmPurchasePage() {
    return <Box
        display={"flex"}
        flexDirection={"column"}
        alignItems={"center"}
    >
        <Typography variant="h2">Thanks for supporting the project!</Typography>
        <Typography>If your new premium/lite slot does not show up contact support in the discord</Typography>
        <BlLink to="/user/premium" variant="contained" color="success">Back to premium slots</BlLink>
    </Box>
}