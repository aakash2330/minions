import { PanelSessionChat } from "./PanelSessionChat";
import { usePanelStore, type PanelContent } from "./stores/panelStore";

export function PanelRenderer() {
  const content = usePanelStore((state) => state.content);

  return renderPanelContent(content);
}

function renderPanelContent(content: PanelContent | null) {
  switch (content?.type) {
    case "session-chat":
      return <PanelSessionChat sessionId={content.sessionId} />;
    default:
      return null;
  }
}
