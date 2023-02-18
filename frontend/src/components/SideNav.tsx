import { Divider, Drawer, List, ListItem, ListItemButton, ListItemText, Toolbar } from "@mui/material";
import { Box } from "@mui/system";
import { ReactNode, useEffect } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { UseSideNavController } from "./SideNavManager";
import { TopBarNavPages } from "./TopNav";

export interface SideNavItem {
    label: string,
    isNavLink: boolean,
    path: string,
    children?: SideNavItem[],
    exact?: boolean
}

export interface SideNavItemMap {
    [key: string]: SideNavItem;
}

const drawerWidth = 250;

export function SideNav<T extends SideNavItemMap>(props: { items: T }) {
    let sideNavController = UseSideNavController();

    useEffect(() => {
        sideNavController.addInstance();
        return function () {
            sideNavController.removeInstance();
        }

        // this is fine even though it warns us
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    let keys = Object.keys(props.items);

    const drawerContents = (
        <>
            <List>
                {keys.map((k) => (
                    <Item item={props.items[k as keyof T]} key={k} />
                ))}
            </List>
        </>
    );

    // we move the top bar pages to the side bar to best utilize the space
    const topBarReplacements = (
        <>
            {TopBarNavPages.map((v, i) => (
                <Item item={{
                    isNavLink: !v.useHref,
                    label: v.label,
                    path: v.path,
                }} key={i} />
            ))}
        </>
    )

    const container = window !== undefined ? () => window.document.body : undefined;

    return <Box
        component="nav"
        sx={{ width: { md: drawerWidth }, flexShrink: { md: 0 } }}
        aria-label="mailbox folders"
    >
        {/* The implementation can be swapped with js to avoid SEO duplication of links. */}
        <Drawer
            container={container}
            variant="temporary"
            open={sideNavController.isOpen}
            onClose={sideNavController.close}
            ModalProps={{
                keepMounted: true, // Better open performance on mobile.
            }}
            sx={{
                display: { xs: 'block', md: 'none' },
                '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
            }}
        >
            <Toolbar >{":)"}</Toolbar>
            <Divider />
            {topBarReplacements}
            <Divider />
            {drawerContents}
        </Drawer>
        <Drawer
            variant="permanent"
            sx={{
                display: { xs: 'none', md: 'block' },
                '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
            }}
            open
        >
            <Toolbar >{":)"}</Toolbar>
            <Divider />
            {drawerContents}
        </Drawer>
    </Box>
}

function Item(props: { item: SideNavItem }) {
    const location = useLocation();
    const isActive = props.item.exact
        ? location.pathname === props.item.path
        : location.pathname.startsWith(props.item.path);

    return <ListItem disablePadding>
        <NavButton item={props.item} selected={isActive}>
            <ListItemText primary={props.item.label} />
        </NavButton>
    </ListItem >
}

function NavButton({ item, children, selected }: { item: SideNavItem, children: ReactNode, selected: boolean }) {
    const navigate = useNavigate();
    let sideNavController = UseSideNavController();

    if (item.isNavLink) {
        return <ListItemButton selected={selected} onClick={() => {
            navigate(item.path);
            sideNavController.close();
        }}>
            {children}
        </ListItemButton>
    }

    return <ListItemButton href={item.path}>{children}</ListItemButton>
}