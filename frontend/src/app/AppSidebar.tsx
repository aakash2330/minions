import {
  Bot,
  Map,
  Settings,
} from "lucide-react";

import {
  AppSection,
  useAppNavigationStore,
} from "@/app/stores/appNavigationStore";
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
  { id: AppSection.World, title: "World", icon: Map },
];

export function AppSidebar() {
  const activeSection = useAppNavigationStore((state) => state.activeSection);
  const setActiveSection = useAppNavigationStore(
    (state) => state.setActiveSection,
  );

  return (
    <Sidebar collapsible="none" className="min-h-svh">
      <SidebarHeader className="px-3 py-3">
        <div className="flex items-center gap-2 rounded-md px-2 py-1.5">
          <div className="grid size-7 place-items-center rounded-md bg-sidebar-primary text-sidebar-primary-foreground">
            <Bot className="size-4" />
          </div>
          <div className="min-w-0">
            <p className="truncate text-sm font-medium">Minions</p>
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
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    isActive={activeSection === item.id}
                    onClick={() => {
                      setActiveSection(item.id);
                    }}
                  >
                    <item.icon />
                    <span>{item.title}</span>
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
              isActive={activeSection === AppSection.Settings}
              onClick={() => {
                setActiveSection(AppSection.Settings);
              }}
            >
              <Settings />
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
