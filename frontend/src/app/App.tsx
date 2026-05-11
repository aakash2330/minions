import { lazy, Suspense } from "react";

import { AppSidebar } from "./AppSidebar";
import { HistoricalConversations } from "@/features/conversations/HistoricalConversations";
import { PhaserGame } from "@/features/world/PhaserGame";
import {
  AppSection,
  useAppNavigationStore,
} from "@/app/stores/appNavigationStore";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";

const MapBuilderPage = lazy(() =>
  import("@/map-builder").then((module) => ({
    default: module.MapBuilderPage,
  })),
);

export function App() {
  const activeSection = useAppNavigationStore((state) => state.activeSection);
  const pathname = window.location.pathname.replace(/\/+$/, "");

  if (pathname === "/workspace/builder") {
    return (
      <div className="dark min-h-screen bg-background text-foreground">
        <TooltipProvider>
          <Suspense fallback={<div className="app-shell">Loading builder...</div>}>
            <MapBuilderPage />
          </Suspense>
        </TooltipProvider>
      </div>
    );
  }

  return (
    <div className="dark min-h-screen bg-background text-foreground">
      <TooltipProvider>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset>
            <main
              className={
                activeSection === AppSection.World
                  ? "app-shell app-shell--world"
                  : "app-shell app-shell--panel"
              }
            >
              {activeSection === AppSection.World ? (
                <PhaserGame />
              ) : (
                <AppSectionContent activeSection={activeSection} />
              )}
            </main>
          </SidebarInset>
        </SidebarProvider>
      </TooltipProvider>
    </div>
  );
}

function AppSectionContent({ activeSection }: { activeSection: AppSection }) {
  if (activeSection === AppSection.Conversations) {
    return <HistoricalConversations />;
  }

  return (
    <section className="mx-auto flex min-h-[calc(100vh-48px)] w-full max-w-5xl items-center justify-center rounded-lg border border-dashed border-border bg-card/40 p-8 text-center">
      <p className="text-sm text-muted-foreground">Nothing to show here yet.</p>
    </section>
  );
}
