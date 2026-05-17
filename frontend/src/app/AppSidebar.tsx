import {
  Bot,
  Map,
  Settings,
} from "lucide-react";
import { NavLink, useLocation } from "react-router-dom";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

const items = [
  { fallbackPath: "/world", segment: "world", title: "World", icon: Map },
];

export function AppSidebar() {
  const { pathname } = useLocation();
  const workspaceBasePath = getWorkspaceBasePath(pathname);

  return (
    <Sidebar collapsible="none" className="min-h-svh">
      <SidebarHeader className="px-3 py-3">
        <div className="flex items-center gap-2 rounded-md px-2 py-1.5">
          <div className="grid size-7 place-items-center rounded-md bg-sidebar-primary text-sidebar-primary-foreground">
            <Bot className="size-4" />
          </div>
          <div className="min-w-0">
            <p className="truncate text-sm font-medium">Sessions</p>
            <p className="truncate text-xs text-muted-foreground">Workshop</p>
          </div>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Game</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.segment}>
                  <SidebarMenuButton
                    asChild
                    isActive={pathname === getItemPath(item, workspaceBasePath)}
                  >
                    <NavLink to={getItemPath(item, workspaceBasePath)}>
                      <item.icon />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="mt-auto">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              asChild
              isActive={pathname === "/settings"}
            >
              <NavLink to="/settings">
                <Settings />
                <span>Settings</span>
              </NavLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}

function getWorkspaceBasePath(pathname: string) {
  const match = pathname.match(/^\/workspace\/([^/]+)\/world$/);

  return match ? `/workspace/${match[1]}` : null;
}

function getItemPath(
  item: (typeof items)[number],
  workspaceBasePath: string | null,
) {
  return workspaceBasePath
    ? `${workspaceBasePath}/${item.segment}`
    : item.fallbackPath;
}
