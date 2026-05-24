import { PanelGlobalChat } from "./PanelGlobalChat";
import { PanelSessionChat } from "./PanelSessionChat";
import {
  PanelContentType,
  usePanelStore,
  type PanelContent,
} from "./stores/panelStore";

export function PanelRenderer() {
  const content = usePanelStore((state) => state.content);

  return renderPanelContent(content);
}

function renderPanelContent(content: PanelContent | null) {
  if (!content) {
    return null;
  }

  switch (content.type) {
    case PanelContentType.GlobalChat:
      return <PanelGlobalChat workspaceId={content.workspaceId} />;
    case PanelContentType.SessionChat:
      return <PanelSessionChat sessionId={content.sessionId} />;
    default:
      return null;
  }
}
