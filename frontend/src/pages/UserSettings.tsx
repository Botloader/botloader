import { useEffect, useRef, useState } from "react";
import { isErrorResponse, PremiumSlot, SessionMeta } from "botloader-common";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { DisplayDateTime } from "../components/DateTime";
import { Panel } from "../components/Panel";
import { useSession } from "../components/Session";
import "./UserSettings.css"
import { useGuilds } from "../components/GuildsProvider";
import { Navigate } from "react-router-dom";

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
        return <Navigate to="/"></Navigate>
    }

    return <div className="user-settings">
        <Panel title="General actions">
            <p>Log out this session</p>
            <AsyncOpButton label="Logout" onClick={() => doLogout()}></AsyncOpButton>
        </Panel>

        <PremiumPanel></PremiumPanel>

        <Panel title="Create API key">
            <CreateApiKeyComponent onCreated={(s) => setAllSessions([...(allSessions || []), s])} />
        </Panel>

        <Panel title="Active sessions">
            <p>Manage your active sessions</p>
            <div className="user-active-sessions">
                {allSessions?.map((elem) => <SessionItem key={elem.created_at} item={elem} />)}
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
        <button className="bl-button" onClick={() => doCreateApiToken()}>Create a new API token</button>
        {status.success ?
            <p>Success!: token: <code>{status.success.token}</code></p> : null}

        {status.error ? <p> Error: <code>{status.error}</code></p> : null}
    </>
}

function PremiumPanel() {
    const session = useSession();
    const [slots, setSlots] = useState<null | undefined | PremiumSlot[]>(undefined);

    async function doFetchSlots() {
        let resp = await session.apiClient.getUserPremiumSlots();
        if (!isErrorResponse(resp)) {
            setSlots(resp);
        } else {
            setSlots(null);
        }
    }

    useEffect(() => {
        doFetchSlots();
    }, [])

    function onChanged() {
        setSlots(undefined);
        doFetchSlots();
    }

    return <Panel>
        <h3>Premium</h3>
        <p>Note: premium offers no benefits at the time of writing</p>
        <p>It might take a minute before your subscription is processed and shows up here.</p>
        <p><a href="https://upgrade.chat/botloader" className="bl-button">Buy premium slots through upgrade.chat</a></p>

        {slots === undefined ?
            <p>Loading...</p> :
            slots === null ? <p>failed fetching slots, refresh the page to try again...</p> :
                slots.map(v => <PremiumSlotComponent key={v.id} slot={v} onChange={onChanged}></PremiumSlotComponent>)
        }
    </Panel>
}

function PremiumSlotComponent(props: { slot: PremiumSlot, onChange: () => any }) {
    const [isChangingGuild, setIsChangingGuild] = useState(false);
    const changeGuildRef = useRef<HTMLSelectElement>(null);
    const session = useSession();

    const guilds = useGuilds();

    let expiresAt = new Date(Date.parse(props.slot.expires_at));

    const attachedGuild = props.slot.attached_guild_id ?
        guilds?.guilds.find(v => v.guild.id === props.slot.attached_guild_id)?.guild.name ?? props.slot.attached_guild_id
        : "Not assigned to a server"

    function setServer() {
        setIsChangingGuild(true);
    }

    async function saveNewServer() {
        let newGuild: string | null = changeGuildRef.current!.value;
        if (newGuild === "none") {
            newGuild = null
        }

        console.log(changeGuildRef.current?.value);
        let resp = await session.apiClient.updatePremiumSlotGuild(props.slot.id + "", newGuild);
        props.onChange();
        if (isErrorResponse(resp)) {
            alert("failed updating guild_id, try again later or contact support");
        }
    }

    return <div className={`premium-slot premium-slot-${props.slot.tier.toLowerCase()}`}>
        <h4>{props.slot.tier === "Premium" ? "👑" : "🍔"}{props.slot.title}</h4>
        <p>Tier: <b>{props.slot.tier}</b>{props.slot.tier === "Premium" ? "👑" : "🍔"}</p>
        <p>Expires at {expiresAt.toLocaleString()}</p>
        <p>State: <b>{props.slot.state}</b></p>
        <p>Attached to: <b>{attachedGuild}</b></p>
        {isChangingGuild ?
            <div className="premium-slot-change-guild">
                <select ref={changeGuildRef}>
                    <option value="none">none</option>
                    {guilds?.guilds.map(v => <option defaultValue={props.slot.attached_guild_id ?? undefined} key={v.guild.id} value={v.guild.id + ""}>{v.guild.name}</option>)}
                </select>
                <AsyncOpButton label="Save" onClick={saveNewServer}></AsyncOpButton>
            </div>
            :
            <button className="bl-button" onClick={setServer}>Set server</button>
        }
    </div>

    // <p>{props.slot.id}</p>
}