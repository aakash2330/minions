import { PanelMinionChat } from "./PanelMinionChat";
import { usePanelStore, type PanelContent } from "./stores/panelStore";

export function PanelRenderer() {
  const content = usePanelStore((state) => state.content);

  return renderPanelContent(content);
}

function renderPanelContent(content: PanelContent | null) {
  switch (content?.type) {
    case "minion-chat":
      return <PanelMinionChat minionId={content.minionId} />;
    default:
      return null;
  }
}
