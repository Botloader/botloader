import { Box } from "@mui/material";

export function CodeBlock({ children }: { children: React.ReactElement }) {
    return <Box
        sx={{
            whiteSpace: "pre-wrap",
            backgroundColor: "rgba(0, 0, 0, 0.5)",
            color: "rgb(247, 85, 85)",
            fontFamily: '"Roboto","Helvetica","Arial",sans-serif'
        }}
    >
        {/* <code> */}
        {children}
        {/* </code> */}
    </Box>
}