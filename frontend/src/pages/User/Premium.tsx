import { Alert, Button, Paper } from "@mui/material";
import { isErrorResponse, PremiumSlot } from "botloader-common";
import { useEffect, useRef, useState } from "react";
import { AsyncOpButton } from "../../components/AsyncOpButton";
import { useGuilds } from "../../components/GuildsProvider";
import { useSession } from "../../components/Session";

export function UserPremiumPage() {
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
        async function innerDoFetchSlots() {
            let resp = await session.apiClient.getUserPremiumSlots();
            if (!isErrorResponse(resp)) {
                setSlots(resp);
            } else {
                setSlots(null);
            }
        }

        innerDoFetchSlots();
    }, [session])

    function onChanged() {
        setSlots(undefined);
        doFetchSlots();
    }

    return <Paper sx={{ padding: 2 }}>
        <h3>Premium</h3>
        <Alert severity="warning">Note: premium offers no benefits at the time of writing</Alert>
        <p>It might take a minute before your subscription is processed and shows up here.</p>
        <Button href="https://upgrade.chat/botloader">Buy premium slots through upgrade.chat</Button>

        {slots === undefined ?
            <p>Loading...</p> :
            slots === null ? <p>failed fetching slots, refresh the page to try again...</p> :
                slots.map(v => <PremiumSlotComponent key={v.id} slot={v} onChange={onChanged}></PremiumSlotComponent>)
        }
    </Paper>
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
}