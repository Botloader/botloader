import { createContext, useContext, useState } from "react";

export function SideNavStateController({ children }: { children: React.ReactNode }) {
    const [isOpen, setOpen] = useState(false);
    const [openInstances, setOpenInstances] = useState(0);

    function modOpenInstances(n: number) {
        console.log("modified");
        setOpenInstances((current) => current + n);
    }

    return <SideNavContext.Provider value={{
        addInstance: () => modOpenInstances(1),
        removeInstance: () => modOpenInstances(-1),
        isOpen: openInstances > 0 && isOpen,
        pageHasSideNav: openInstances > 0,
        toggle: () => setOpen((current) => !current),
        close: () => setOpen(false),
        open: () => setOpen(true),
        instances: openInstances,
    }}>
        {children}
    </SideNavContext.Provider>

}

export const SideNavContext = createContext<SideNavContextData | undefined>(undefined);

export interface SideNavContextData {
    isOpen: boolean,
    pageHasSideNav: boolean,
    toggle: () => unknown,
    close: () => unknown,
    open: () => unknown,
    addInstance: () => unknown,
    removeInstance: () => unknown,
    instances: number,
}

export function UseSideNavController() {
    return useContext(SideNavContext)!;
}