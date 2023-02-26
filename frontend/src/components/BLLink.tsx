import { Button, ButtonProps } from "@mui/material";
import { Link } from "react-router-dom";

export function BlLink(props: { to: string, skipClientRouting?: boolean } & Pick<ButtonProps, "color" | "children" | "variant" | "sx" | "disabled">) {
    return <Button {...props} component={Link} reloadDocument={props.skipClientRouting} sx={{}}>{props.children}</Button>
}