import { useEffect, useState } from "react";
import { Redirect } from "react-router";
import { isErrorResponse, SessionMeta } from "botloader-common";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { DisplayDateTime } from "../components/DateTime";
import { Panel } from "../components/Panel";
import { useSession } from "../components/Session";
import "./UserSettings.css"

export function UserSettingsPage() {

    const session = useSession();
    let [allSessions, setAllSessions] = useState<SessionMeta[] | undefined | null>(undefined);

    const [signedOut, setSignedOut] = useState<boolean>(false);

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
        await session.apiClient.logout();
        setSignedOut(true);
    }

    async function clearAllSessions() {
        await session.apiClient.deleteAllSessions();
        setSignedOut(true);
    }

    if (signedOut) {
        return <Redirect to="/"></Redirect>
    }

    return <div className="user-settings">
        <Panel title="General actions">
            <p>Log out this session</p>
            <AsyncOpButton label="Logout" onClick={() => doLogout()}></AsyncOpButton>
        </Panel>

        <Panel title="Create API key">
            <CreateApiKeyComponent onCreated={(s) => setAllSessions([...(allSessions || []), s])} />
        </Panel>

        <Panel title="Active sessions">
            <p>Manage your active sessions</p>
            <div className="user-active-sessions">
                {allSessions?.map((elem) => <SessionItem key={elem.token} item={elem} />)}
            </div>
            <br />
            <AsyncOpButton label="Clear all sessions (including logging you out from this one)" onClick={() => clearAllSessions()} className="danger"></AsyncOpButton>
        </Panel>
    </div>
}

function SessionItem(props: { item: SessionMeta }) {
    return <>
        <div className="session-kind">{props.item.kind}</div>
        <div className="session-created_at"><DisplayDateTime dt={props.item.created_at} /></div>
    </>
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

    if (status.creating) {
        return <p>Creating...</p>
    }

    return <>
        <p>WARNING: This will display the key on screen, anyone with the key can log into your account</p>
        <button onClick={() => doCreateApiToken()}>Create a new API token</button>
        {status.success ?
            <p>Success!: token: <code>{status.success.token}</code></p> : null}

        {status.error ? <p> Error: <code>{status.error}</code></p> : null}
    </>
}