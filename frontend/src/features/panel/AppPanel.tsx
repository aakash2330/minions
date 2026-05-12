import { Panel } from "./Panel";
import { PanelRenderer } from "./PanelRenderer";
import { usePanelStore } from "./stores/panelStore";

export function AppPanel() {
  const isOpen = usePanelStore((state) => state.isOpen);
  const setOpen = usePanelStore((state) => state.setOpen);

  return (
    <Panel.Root open={isOpen} onOpenChange={setOpen}>
      <Panel.Content>
        <PanelRenderer />
      </Panel.Content>
    </Panel.Root>
  );
}
