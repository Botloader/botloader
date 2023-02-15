import { Alert, Button, Card, CardActions, CardContent, Paper, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Typography } from "@mui/material";
import { Stack } from "@mui/system";
import { isErrorResponse, SessionMeta } from "botloader-common";
import { useEffect, useState } from "react";
import { AsyncOpButton } from "../../components/AsyncOpButton";
import { DisplayDateTime } from "../../components/DateTime";
import { useSession } from "../../components/Session";
import { sessionManager } from "../../util/SessionManager";

export function UserGeneralPage() {
    const session = useSession();
    let [allSessions, setAllSessions] = useState<SessionMeta[] | undefined | null>(undefined);

    useEffect(() => {
        async function fetchSessions() {
            const resp = await session.apiClient.getAllSessions();
            if (isErrorResponse(resp)) {
                setAllSessions(null);
            } else {
                setAllSessions(resp);
            }
        }

        fetchSessions();
    }, [session])


    async function doLogout() {
        await sessionManager.logout();
    }

    return <Stack spacing={1}>
        <Paper sx={{ padding: 1 }}>
            <AsyncOpButton label="Sign out" onClick={() => doLogout()}></AsyncOpButton>
        </Paper>

        <CreateApiKeyComponent onCreated={(s) => setAllSessions([...(allSessions || []), s])} />

        <Sessions sessions={allSessions ?? []} />
    </Stack>
}

type CreateApiTokenProps = {
    onCreated?: (s: SessionMeta) => void,
}

interface TokenStatus {
    creating: boolean,

    success?: SessionMeta,
    error?: string,
}

function CreateApiKeyComponent(props: CreateApiTokenProps) {
    let session = useSession();

    const [status, setStatus] = useState<TokenStatus>({
        creating: false,
    });

    async function doCreateApiToken() {
        setStatus({ creating: true })
        let resp = await session.apiClient.createApiToken();
        if (isErrorResponse(resp)) {
            setStatus({
                creating: false,
                error: JSON.stringify(resp),
            })
        } else {
            setStatus({
                creating: false,
                success: resp,
            })

            if (props.onCreated) {
                props.onCreated(resp);
            }
        }
    }

    return <Card>
        <CardContent>
            <Typography variant="h5" mb={1}>Create API Keys</Typography>
            <Alert severity="warning">WARNING: This will display the key on screen, anyone with the key can log into your account.</Alert>
            <Button
                disabled={status.creating}
                onClick={() => doCreateApiToken()}>
                {status.creating ? "Creating..." : "Create a new API key"}
            </Button>
            {status.success ?
                <p>Success!: token: <code>{status.success.token}</code></p> : null}

            {status.error ? <p> Error: <code>{status.error}</code></p> : null}
        </CardContent>
    </Card>
}

function Sessions({ sessions }: { sessions: SessionMeta[] }) {
    async function clearAllSessions() {
        await sessionManager.logoutAllSessions();
    }

    return <Card>
        <CardContent>
            <Typography variant="h5" mb={1}>
                Active logins
            </Typography>
            <Paper elevation={2}>
                <TableContainer>
                    <Table>
                        <TableHead>
                            <TableRow>
                                <TableCell>Login Type</TableCell>
                                <TableCell>Date</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {sessions?.map((elem, i) => <TableRow key={i}>
                                <TableCell>{elem.kind}</TableCell>
                                <TableCell><DisplayDateTime dt={elem.created_at} /></TableCell>
                            </TableRow>)}
                        </TableBody>
                    </Table>
                </TableContainer>
            </Paper>
        </CardContent>
        <CardActions>
            <AsyncOpButton label="Clear all logins/api keys (also logs you out)" onClick={() => clearAllSessions()} className="danger"></AsyncOpButton>
        </CardActions>
    </Card>

}