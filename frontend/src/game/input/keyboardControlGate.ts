type CanUseGameKeyboardInputOptions = {
  disabled?: boolean;
  activeElement?: Element | null;
};

const EDITABLE_SELECTOR = [
  "input",
  "textarea",
  "select",
  "[contenteditable='']",
  "[contenteditable='true']",
  "[role='textbox']",
].join(",");

export type CanUseKeyboardInput = () => boolean;

export function canUseGameKeyboardInput({
  disabled = false,
  activeElement = document.activeElement,
}: CanUseGameKeyboardInputOptions = {}) {
  if (disabled) {
    return false;
  }

  return !isEditableElement(activeElement);
}

function isEditableElement(element: Element | null) {
  if (!element || !element.matches(EDITABLE_SELECTOR)) {
    return false;
  }

  if (element instanceof HTMLInputElement) {
    return !element.disabled && !element.readOnly;
  }

  if (
    element instanceof HTMLTextAreaElement ||
    element instanceof HTMLSelectElement
  ) {
    return !element.disabled;
  }

  return true;
}
