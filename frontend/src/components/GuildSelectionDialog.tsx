import Avatar from '@mui/material/Avatar';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import DialogTitle from '@mui/material/DialogTitle';
import Dialog from '@mui/material/Dialog';
import { BotGuild } from 'botloader-common';
import { GuildIcon } from './GuildIcon';


export interface GuildSelectionDialogProps {
    open: boolean;
    selectedValue?: string;
    guilds: BotGuild[],
    onClose: (value: BotGuild | null) => void;
}


export function GuildSelectionDialog(props: GuildSelectionDialogProps) {
    const { onClose, open } = props;

    const handleClose = () => {
        onClose(null);
    };

    const handleListItemClick = (value: BotGuild) => {
        onClose(value);
    };

    return (
        <Dialog onClose={handleClose} open={open}>
            <DialogTitle>Select a server</DialogTitle>
            <List sx={{ pt: 0 }}>
                {props.guilds.map((g) => (
                    <ListItem disableGutters key={g.guild.id}>
                        <ListItemButton onClick={() => handleListItemClick(g)} key={g.guild.id}>
                            <ListItemAvatar>
                                <Avatar sx={{ color: "white" }}>
                                    <GuildIcon guild={g.guild} />
                                </Avatar>
                            </ListItemAvatar>
                            <ListItemText primary={g.guild.name} />
                        </ListItemButton>
                    </ListItem>
                ))}
            </List>
        </Dialog>
    );
}