import { AppSidebar } from "./AppSidebar";
import { PhaserGame } from "./PhaserGame";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";

export function App() {
  return (
    <div className="dark min-h-screen bg-background text-foreground">
      <TooltipProvider>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset>
            <main className="app-shell">
              <PhaserGame />
            </main>
          </SidebarInset>
        </SidebarProvider>
      </TooltipProvider>
    </div>
  );
}
