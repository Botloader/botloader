import { Button, ButtonProps, Paper } from "@mui/material";
import { isErrorResponse, PremiumSlot, PremiumSlotTier } from "botloader-common";
import { useRef } from "react";
import { AsyncOpButton } from "../../../components/AsyncOpButton";
import { GuildsGuard, useGuilds } from "../../../modules/guilds/GuildsProvider";
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
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../../components/FetchData";
import { UseNotifications } from "../../../components/Notifications";
import { useSession } from "../../../modules/session/useSession";
import { Loading } from "../../../components/Loading";

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
    const session = useSession()
    const [isCallingApi, setIsCallingApi] = React.useState(false)
    const { value: slots } = useFetchedDataBehindGuard(slotsContext);
    const notifications = UseNotifications()

    async function startCheckout(tier: PremiumSlotTier) {
        let resp = await session.apiClient.createStripeCheckoutSession(tier)
        if (isErrorResponse(resp)) {
            notifications.push({
                class: "error",
                message: "Something went wrong while creating checkout session, join the discord server for more info."
            })
        } else {
            window.location.href = resp.url
        }
    }

    async function customerPortal() {
        let resp = await session.apiClient.createStripeCustomerPortalSession()
        if (isErrorResponse(resp)) {
            notifications.push({
                class: "error",
                message: "Something went wrong while creating the stripe customer portal session, join the discord server for more info."
            })
        } else {
            window.location.href = resp.url
        }
    }

    return (<Box p={3}>
        <Typography variant="h4" mb={2}>Premium</Typography>

        <Paper sx={{ padding: 2, display: "flex", gap: 2, flexDirection: "column" }}>
            <Box>
                <Typography variant="h6">Premium / Lite offers the following benefits</Typography>
            </Box>
            <Box>
                <Typography>Priority to the worker pool when handling events</Typography>
                <Typography color={"text.secondary"}>If the bot is experiencing load then your sever will be given priority over other servers when handling events, leading to more reliable operation</Typography>
            </Box>
            <Box>
                <Typography>Access to Premium/Lite dedicated worker pool</Typography>
                <Typography color={"text.secondary"}>In addition to the priority mentioned above your server will get access to handling events on reserved workers, again leading to more reliable  operation</Typography>
            </Box>
            <Box>
                <Typography >Higher limits</Typography>
                <Typography color={"text.secondary"}>You get higher storage, ratelimits, task limits and so on, the exact limits are still being tuned so ask in the support server for more information.</Typography>
            </Box>

            <Box display={"flex"} flexDirection={"row"} gap={1}>
                <Button
                    color="success"
                    variant="contained"
                    disabled={isCallingApi}
                    onClick={() => {
                        if (isCallingApi) {
                            return
                        }
                        setIsCallingApi(true)
                        startCheckout("Lite").finally(() => setIsCallingApi(false))
                    }}
                >
                    Subscribe to Lite for $3/month
                </Button>
                <Button
                    color="success"
                    variant="contained"
                    disabled={isCallingApi}
                    onClick={() => {
                        if (isCallingApi) {
                            return
                        }
                        setIsCallingApi(true)
                        startCheckout("Premium").finally(() => setIsCallingApi(false))
                    }}
                >
                    Subscribe to Premium for $5/month
                </Button>
            </Box>
            <Box >
                <Button
                    color="primary"
                    variant="contained"
                    disabled={isCallingApi}
                    onClick={() => {
                        if (isCallingApi) {
                            return
                        }
                        setIsCallingApi(true)
                        customerPortal().finally(() => setIsCallingApi(false))
                    }}
                >
                    Manage existing subscriptions
                </Button>
            </Box>
        </Paper>

        <Typography mt={1}>It might take a minute before your subscription is processed and shows up here.</Typography>



        <GuildsGuard>
            <Box display="flex" mt={1} gap={1} flexWrap="wrap">
                {slots === undefined ?
                    <Loading /> :
                    slots === null ? <p>failed fetching slots, refresh the page to try again...</p> :
                        slots.map(v => <PremiumSlotComponent key={v.id} slot={v}></PremiumSlotComponent>)
                }
            </Box>
        </GuildsGuard>
    </Box>)
}

function PremiumSlotComponent(props: { slot: PremiumSlot }) {
    const { reload } = useFetchedDataBehindGuard(slotsContext);
    const notifications = UseNotifications();
    const changeGuildRef = useRef<HTMLSelectElement>(null);
    const session = useSession();

    const guilds = useGuilds()!;

    const [expanded, setExpanded] = React.useState(false);

    const handleExpandClick = () => {
        setExpanded((current) => !current);
    };

    const expiresAt = new Date(Date.parse(props.slot.expires_at));

    const attachedGuild = props.slot.attached_guild_id ?
        guilds?.value?.all.find(v => v.guild.id === props.slot.attached_guild_id)?.guild.name ?? props.slot.attached_guild_id
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
            notifications.push({
                class: "error",
                message: "Failed updating premium slot: " + (resp.response?.description ?? "unknown error")
            })
        } else {
            notifications.push({
                class: "success",
                message: "Premium slot updated"
            })
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
                        {guilds.value?.all.filter((v) => v.connected)
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
