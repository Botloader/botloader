import { Button } from "@mui/material";
import { useState } from "react"

type Props = {
    label: string,
    onClick: () => any,
    className?: string,
    disabled?: boolean,
}

export function AsyncOpButton(props: Props) {
    const [status, setStatus] = useState<boolean>(false);


    async function doOp() {
        setStatus(true);
        await props.onClick();
        setStatus(false);
    }

    return <Button
        disabled={props.disabled || status}
        onClick={() => doOp()}
        className={(props.className ?? "")}>
        {props.label}
    </Button>
}