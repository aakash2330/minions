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
    case PanelContentType.SessionChat:
      return <PanelSessionChat sessionId={content.sessionId} />;
    default:
      return null;
  }
}
