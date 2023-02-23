import { Alert, Button, ButtonProps } from "@mui/material";
import { isErrorResponse, PremiumSlot } from "botloader-common";
import { useRef } from "react";
import { AsyncOpButton } from "../../components/AsyncOpButton";
import { GuildsGuard, useGuilds } from "../../components/GuildsProvider";
import { useSession } from "../../components/Session";
import * as React from 'react';
import { styled } from '@mui/material/styles';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';
import CardActions from '@mui/material/CardActions';
import Collapse from '@mui/material/Collapse';
import Typography from '@mui/material/Typography';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { Box } from "@mui/system";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../components/FetchData";

let slotsContext = createFetchDataContext<PremiumSlot[]>();

export function UserPremiumPage() {
    const session = useSession();

    async function fetchSlots() {
        const resp = await session.apiClient.getUserPremiumSlots();
        if (!isErrorResponse(resp)) {
            resp.sort((a, b) => a.id - b.id);
        }
        return resp;
    }

    return <FetchDataGuarded context={slotsContext} loader={fetchSlots}>
        <InnerPage />
    </FetchDataGuarded>
}

export function InnerPage() {
    const { value: slots } = useFetchedDataBehindGuard(slotsContext);


    return (<Box p={3}>
        <Typography variant="h4">Premium</Typography>
        <Alert severity="warning">Note: premium offers no benefits at the time of writing</Alert>
        <p>It might take a minute before your subscription is processed and shows up here.</p>
        <Button href="https://upgrade.chat/botloader">Buy premium slots through upgrade.chat</Button>

        <GuildsGuard>
            <Box display="flex" mt={1} gap={1} flexWrap="wrap">
                {slots === undefined ?
                    <p>Loading...</p> :
                    slots === null ? <p>failed fetching slots, refresh the page to try again...</p> :
                        slots.map(v => <PremiumSlotComponent key={v.id} slot={v}></PremiumSlotComponent>)
                }
            </Box>
        </GuildsGuard>
    </Box>)
}

function PremiumSlotComponent(props: { slot: PremiumSlot }) {
    const { reload } = useFetchedDataBehindGuard(slotsContext);

    const changeGuildRef = useRef<HTMLSelectElement>(null);
    const session = useSession();

    const guilds = useGuilds()!;

    const [expanded, setExpanded] = React.useState(false);

    const handleExpandClick = () => {
        setExpanded((current) => !current);
    };

    const expiresAt = new Date(Date.parse(props.slot.expires_at));

    const attachedGuild = props.slot.attached_guild_id ?
        guilds?.all.find(v => v.guild.id === props.slot.attached_guild_id)?.guild.name ?? props.slot.attached_guild_id
        : "None"


    async function saveNewServer() {
        let newGuild: string | null = changeGuildRef.current!.value;
        if (newGuild === "none") {
            newGuild = null
        }

        console.log(changeGuildRef.current?.value);
        let resp = await session.apiClient.updatePremiumSlotGuild(props.slot.id + "", newGuild);
        reload();
        if (isErrorResponse(resp)) {
            alert("failed updating slot, try again later or contact support");
        }
    }

    return <Card sx={{ width: 250 }}>
        <CardHeader
            title={props.slot.tier}
            subheader={props.slot.source + " - " + props.slot.title}
        />
        <CardContent>
            <Typography variant="body2" color="text.secondary">
                {props.slot.message}
            </Typography>
            <Typography variant="body2" color="text.secondary" mt={1}>
                Expires at <b>{expiresAt.toLocaleString()}</b>
            </Typography>
            <Typography color="text.secondary">State: <b>{props.slot.state}</b></Typography>
            <Typography color="text.secondary">Attached to: <b>{attachedGuild}</b></Typography>
        </CardContent>
        <CardActions disableSpacing>
            <ExpandMore
                expand={expanded}
                onClick={handleExpandClick}
                aria-expanded={expanded}
                aria-label="show more"
            >
                <ExpandMoreIcon />
            </ExpandMore>
        </CardActions>
        <Collapse in={expanded} timeout="auto" unmountOnExit>
            <CardContent>
                <div className="premium-slot-change-guild">
                    <select ref={changeGuildRef}>
                        <option value="none">none</option>
                        {guilds.all.filter((v) => v.connected)
                            .map(v => <option defaultValue={props.slot.attached_guild_id ?? undefined} key={v.guild.id} value={v.guild.id + ""}>{v.guild.name}</option>)}
                    </select>
                    <AsyncOpButton label="Save" onClick={saveNewServer}></AsyncOpButton>
                </div>
            </CardContent>
        </Collapse>
    </Card>
}



interface ExpandMoreProps extends ButtonProps {
    expand: boolean;
}

const ExpandMore = styled((props: ExpandMoreProps) => {
    const { expand, ...other } = props;
    return <Button {...other}>Set Server</Button>;
})(({ theme, expand }) => ({
    transform: !expand ? 'rotate(0deg)' : 'rotate(180deg)',
    transition: theme.transitions.create('transform', {
        duration: theme.transitions.duration.shortest,
    }),
}));
