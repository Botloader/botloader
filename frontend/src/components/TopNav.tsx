import { Link } from "react-router-dom";
import { BuildConfig } from "../BuildConfig";
import { userAvatarUrl } from "./Util";
import { GuildIcon } from "./GuildIcon";
import { AppBar, Paper } from "@mui/material";
import * as React from 'react';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Menu from '@mui/material/Menu';
import MenuIcon from '@mui/icons-material/Menu';
import Container from '@mui/material/Container';
import Avatar from '@mui/material/Avatar';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import { UseSideNavController } from "./SideNavManager";
import { BlLink } from "./BLLink";
import { Breadcrumbs } from "./BreadCrumbs";
import { useSession } from "../modules/session/useSession";
import { useCurrentGuild } from "../modules/guilds/CurrentGuild";

export const TopBarNavPages = [{
    label: "Docs",
    path: "/docs/",
    useHref: true,
}, {
    label: "Guides",
    path: "/book/",
    useHref: true,
}, {
    label: "News",
    path: "/news",
    useHref: false,
}, {
    label: "Plugins",
    path: "/plugins",
    useHref: false,
}]

export function TopNav() {
    let sideNavController = UseSideNavController();

    let session = useSession();
    let currentGuild = useCurrentGuild();

    const [anchorElNav, setAnchorElNav] = React.useState<null | HTMLElement>(null);

    const handleOpenNavMenu = (event: React.MouseEvent<HTMLElement>) => {
        if (sideNavController.pageHasSideNav) {
            sideNavController.open()
        } else {
            setAnchorElNav(event.currentTarget);
        }
    };

    const handleCloseNavMenu = () => {
        setAnchorElNav(null);
    };

    // const handleClickNavMenuItem = (page: typeof TopBarNavPages[number]) => {
    //     handleCloseNavMenu();

    //     if (!page.useHref) {
    //         navigate(page.path);
    //     }
    // };

    const drawerWidth = sideNavController.pageHasSideNav ? 250 : 0;

    return (<>
        <AppBar position="static"
            sx={{
                width: { md: `calc(100% - ${drawerWidth}px)` },
                ml: { md: `${drawerWidth}px` },
            }}
        >
            <Container maxWidth="xl">
                <Toolbar disableGutters>
                    {/* <AdbIcon sx={{ display: { xs: 'none', md: 'flex' }, mr: 1 }} /> */}
                    <Typography
                        variant="h6"
                        noWrap
                        component="a"
                        href="/"
                        sx={{
                            mr: 2,
                            display: { xs: 'none', md: 'flex' },
                            fontWeight: 700,
                            color: 'inherit',
                            textDecoration: 'none',
                        }}
                    >
                        Botloader
                    </Typography>

                    <Box sx={{ flexGrow: 1, display: { xs: 'flex', md: 'none' } }}>
                        <IconButton
                            size="large"
                            aria-label="account of current user"
                            aria-controls="menu-appbar"
                            aria-haspopup="true"
                            onClick={handleOpenNavMenu}
                            color="inherit"
                        >
                            <MenuIcon />
                        </IconButton>
                        <Menu
                            id="menu-appbar"
                            anchorEl={anchorElNav}
                            anchorOrigin={{
                                vertical: 'bottom',
                                horizontal: 'left',
                            }}
                            keepMounted
                            transformOrigin={{
                                vertical: 'top',
                                horizontal: 'left',
                            }}
                            open={Boolean(anchorElNav) && !sideNavController.pageHasSideNav}
                            onClose={handleCloseNavMenu}
                            sx={{
                                display: { xs: 'block', md: 'none' },
                            }}
                        >
                            {TopBarNavPages.map((page) => (
                                <MenuItem
                                    key={page.label}
                                    component={Link}
                                    to={page.path}
                                    reloadDocument={page.useHref}
                                    target={page.useHref ? "_blank" : undefined}
                                >
                                    <Typography textAlign="center">{page.label}</Typography>
                                </MenuItem>
                            ))}
                        </Menu>
                    </Box>
                    <Typography
                        variant="h5"
                        noWrap
                        component="a"
                        href=""
                        sx={{
                            mr: 2,
                            display: { xs: 'flex', md: 'none' },
                            flexGrow: 1,
                            fontWeight: 700,
                            color: 'inherit',
                            textDecoration: 'none',
                        }}
                    >
                        Botloader
                    </Typography>
                    <Box sx={{ flexGrow: 1, display: { xs: 'none', md: 'flex' } }}>
                        {TopBarNavPages.map((page) => (
                            <BlLink
                                key={page.label}
                                sx={{ my: 2, color: 'white', display: 'block' }}
                                to={page.path}
                                skipClientRouting={page.useHref}
                                newTab={page.useHref}
                            >
                                {page.label}
                            </BlLink>
                        ))}
                    </Box>

                    <Box sx={{ flexGrow: 0 }}>
                        {currentGuild ?
                            <Tooltip title="Change server" >
                                <IconButton component={Link} to="/servers">
                                    <GuildIcon size={40} guild={currentGuild.guild}></GuildIcon>
                                </IconButton>
                            </Tooltip>
                            : session.user ?
                                <BlLink to={"/servers"} sx={{ mr: 1 }}>Servers</BlLink>
                                // Don't show the pick server button if were not logged in
                                : null}

                        {session.user ?
                            <>
                                <Tooltip title="User Settings">
                                    <IconButton component={Link} to={"/user/general"}>
                                        <Avatar alt={session.user.username} src={userAvatarUrl(session.user, 64)} />
                                    </IconButton>
                                </Tooltip>
                            </> :
                            <Tooltip title="Log in">
                                <BlLink to={BuildConfig.botloaderApiBase + "/login"} skipClientRouting>Sign in</BlLink>
                            </Tooltip>}
                    </Box>
                </Toolbar>
            </Container>
        </AppBar >
        <Paper sx={{
            padding: 1,
            width: { md: `calc(100% - ${drawerWidth}px)` },
            ml: { md: `${drawerWidth}px` },
        }}>
            <Breadcrumbs />
        </Paper>
    </>
    );
}
