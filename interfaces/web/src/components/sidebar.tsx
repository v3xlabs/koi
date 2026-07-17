
import { AccountSidebar } from "./account/sidebar";
import { SidebarLeft } from "./sidebar/left";

export const Sidebar = () => (
    <div class="flex h-full">
        <SidebarLeft />
        <AccountSidebar />
    </div>
);
