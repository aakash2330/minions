import {
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";

import { AppSidebar } from "./AppSidebar";
import { WorldPage } from "@/features/world/WorldPage";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
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
              path="sessions"
              element={<Navigate replace to="/world" />}
            />
            <Route path="settings" element={<EmptySection />} />
            <Route path="*" element={<Navigate replace to="/world" />} />
          </Route>
        </Routes>
      </TooltipProvider>
    </div>
  );
}

function AppLayout() {
  const { pathname } = useLocation();
  const isWorldRoute = pathname === "/world";

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <main
          className={
            isWorldRoute
              ? "app-shell app-shell--world"
              : "app-shell app-shell--panel"
          }
        >
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
