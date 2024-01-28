import { Button, ButtonProps } from "@mui/material";
import { Link } from "react-router-dom";

export function BlLink(props: {
    to: string,
    skipClientRouting?: boolean
    newTab?: boolean
    fullWidth?: boolean
} & Pick<ButtonProps, "color" | "children" | "variant" | "sx" | "disabled">) {
    return <Button
        {...props}
        component={Link}
        reloadDocument={props.skipClientRouting}
        sx={{}}
        target={props.newTab ? "_blank" : undefined}
        fullWidth={props.fullWidth}
    >
        {props.children}
    </Button >
}