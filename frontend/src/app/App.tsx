import { Navigate, Outlet, Route, Routes } from "react-router-dom";

import { AppSidebar } from "./AppSidebar";
import { WorldPage } from "@/features/world/WorldPage";
import { WorkspacesPage } from "@/features/workspaces/Page";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppPanel } from "@/features/panel/AppPanel";

export function App() {
  return (
    <div className="dark min-h-screen bg-background text-foreground">
      <TooltipProvider>
        <Routes>
          <Route element={<AppLayout />}>
            <Route index element={<Navigate replace to="/world" />} />
            <Route path="world" element={<WorldPage />} />
            <Route
              path="workspace/:workspaceId"
              element={<Navigate replace to="world" />}
            />
            <Route path="workspace/:workspaceId/world" element={<WorldPage />} />
            <Route path="workspaces" element={<WorkspacesPage />} />
            <Route path="settings" element={<EmptySection />} />
            <Route path="*" element={<Navigate replace to="/world" />} />
          </Route>
        </Routes>
        <Toaster />
      </TooltipProvider>
    </div>
  );
}

function AppLayout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <main className="app-shell">
          <Outlet />
        </main>
      </SidebarInset>
      <AppPanel />
    </SidebarProvider>
  );
}

function EmptySection() {
  return (
    <section className="mx-auto flex min-h-[calc(100vh-48px)] w-full max-w-5xl items-center justify-center rounded-lg border border-dashed border-border bg-card/40 p-8 text-center">
      <p className="text-sm text-muted-foreground">Nothing to show here yet.</p>
    </section>
  );
}
