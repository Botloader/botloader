// import { Link } from "react-router-dom";
// import { BotGuild, User } from "botloader-common";
// import { BuildConfig } from "../BuildConfig";
import { NavLink } from "react-router-dom";
// import { useSession } from "./Session";
import "./SideNav.css"
// import { guildIconUrl, userAvatarUrl } from "./Util";


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


// export type SideNavItemMap<T extends string> = Record<T, SideNavItem>;


export function SideNav<T extends SideNavItemMap>(props: { items: T, activePage?: keyof T }) {

    let keys = Object.keys(props.items);

    return <nav className="side-nav">
        <ul className="side-nav-main">
            {keys.map(k => <li key={k}><Item item={props.items[k as keyof T]} expanded={props.activePage === k} itemName={k}></Item></li>)}
        </ul>
    </nav >
}

function Item(props: { itemName: string, item: SideNavItem, expanded: boolean }) {
    let footer = null;
    if (props.expanded && props.item.children) {
        footer = <ul className="side-nav-sub">
            {props.item.children.map(v => <li key={v.path}><Item item={v} expanded={false} itemName=""></Item></li>)}
        </ul>;
    }

    if (props.item.isNavLink) {
        return <><NavLink to={props.item.path} exact={props.item.exact}>{props.item.label}</NavLink>{footer}</>
    }

    return <><a href={props.item.path}>{props.item.label}</a>{footer}</>
}