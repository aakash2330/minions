import { Button } from "@/components/ui/button";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { AtSignIcon, CheckIcon, SendHorizontalIcon, XIcon } from "lucide-react";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import { Controller, useForm } from "react-hook-form";

export type SessionMentionOption = {
  sessionId: string;
  name: string;
  kind: string;
};

type SessionMessageSubmitButtonProps = {
  isApprovalRequestPending: boolean;
  isApprovalResponsePending: boolean;
  mentionOptions?: SessionMentionOption[];
  onAccept: () => void;
  onDecline: () => void;
  onPromptSubmit: ({ prompt }: { prompt: string }) => void;
};

type SessionMessageFormValues = {
  prompt: string;
};

type ActiveMention = {
  markerIndex: number;
  endIndex: number;
  query: string;
};

export function SessionMessageSubmitButton({
  isApprovalRequestPending,
  isApprovalResponsePending,
  mentionOptions = [],
  onPromptSubmit,
  onAccept,
  onDecline,
}: SessionMessageSubmitButtonProps) {
  const form = useForm<SessionMessageFormValues>({
    defaultValues: {
      prompt: "",
    },
  });
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [cursorIndex, setCursorIndex] = useState(0);
  const [selectedMentionIndex, setSelectedMentionIndex] = useState(0);
  const rawPrompt = form.watch("prompt");
  const prompt = rawPrompt.trim();
  const activeMention = useMemo(
    () => findActiveMention(rawPrompt, cursorIndex),
    [rawPrompt, cursorIndex],
  );
  const activeMentionKey = activeMention
    ? `${activeMention.markerIndex}:${activeMention.query}`
    : "";
  const filteredMentionOptions = useMemo(() => {
    if (!activeMention) {
      return [];
    }

    return mentionOptions
      .filter((option) => mentionOptionMatches(option, activeMention.query))
      .slice(0, 6);
  }, [activeMention, mentionOptions]);
  const showMentionOptions = filteredMentionOptions.length > 0;
  const selectedMentionOptionIndex = Math.min(
    selectedMentionIndex,
    filteredMentionOptions.length - 1,
  );
  const selectedMentionOption =
    filteredMentionOptions[selectedMentionOptionIndex];

  useEffect(() => {
    setSelectedMentionIndex(0);
  }, [activeMentionKey]);

  function onSubmit(values: SessionMessageFormValues) {
    const prompt = values.prompt.trim();
    if (!prompt) return;
    onPromptSubmit({ prompt });
    form.reset();
  }

  function onMentionSelect(option: SessionMentionOption) {
    const currentPrompt = form.getValues("prompt");
    const mention = findActiveMention(currentPrompt, cursorIndex);

    if (!mention) {
      return;
    }

    const replacement = `@${option.sessionId} `;
    const nextPrompt = `${currentPrompt.slice(0, mention.markerIndex)}${replacement}${currentPrompt
      .slice(mention.endIndex)
      .replace(/^ /, "")}`;
    const nextCursorIndex = mention.markerIndex + replacement.length;

    form.setValue("prompt", nextPrompt, {
      shouldDirty: true,
      shouldTouch: true,
      shouldValidate: true,
    });
    setCursorIndex(nextCursorIndex);
    setSelectedMentionIndex(0);
    window.requestAnimationFrame(() => {
      inputRef.current?.focus();
      inputRef.current?.setSelectionRange(nextCursorIndex, nextCursorIndex);
    });
  }

  function onPromptKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (!showMentionOptions) {
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      setSelectedMentionIndex((index) =>
        Math.min(index + 1, filteredMentionOptions.length - 1),
      );
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      setSelectedMentionIndex((index) => Math.max(index - 1, 0));
      return;
    }

    if (
      (event.key === "Enter" || event.key === "Tab") &&
      selectedMentionOption
    ) {
      event.preventDefault();
      onMentionSelect(selectedMentionOption);
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      setCursorIndex(-1);
    }
  }

  function updateCursorFromInput(input: HTMLInputElement) {
    setCursorIndex(input.selectionStart ?? input.value.length);
  }

  if (isApprovalRequestPending) {
    return (
      <div className="flex shrink-0 items-center justify-end gap-2 border-t border-border bg-popover p-4">
        <Button
          disabled={isApprovalResponsePending}
          onClick={onDecline}
          size="sm"
          type="button"
          variant="outline"
        >
          <XIcon className="size-3.5" />
          Decline
        </Button>
        <Button
          disabled={isApprovalResponsePending}
          onClick={onAccept}
          size="sm"
          type="button"
        >
          <CheckIcon className="size-3.5" />
          {isApprovalResponsePending ? "Responding" : "Approve"}
        </Button>
      </div>
    );
  }

  return (
    <form
      className="shrink-0 border-t border-border bg-popover p-4"
      onSubmit={form.handleSubmit(onSubmit)}
    >
      <div className="relative flex items-start gap-2">
        {showMentionOptions && (
          <div
            aria-label="Character mentions"
            className="absolute bottom-full left-0 right-10 z-10 mb-2 max-h-44 overflow-y-auto rounded-lg border border-border bg-popover p-1 shadow-lg"
            role="listbox"
          >
            {filteredMentionOptions.map((option, index) => (
              <button
                aria-selected={index === selectedMentionOptionIndex}
                className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm outline-none hover:bg-muted aria-selected:bg-muted"
                key={option.sessionId}
                onMouseDown={(event) => {
                  event.preventDefault();
                  onMentionSelect(option);
                }}
                role="option"
                type="button"
              >
                <AtSignIcon className="size-3.5 text-muted-foreground" />
                <span className="min-w-0 flex-1 truncate">{option.name}</span>
                <span className="shrink-0 text-xs text-muted-foreground">
                  {option.kind}
                </span>
              </button>
            ))}
          </div>
        )}
        <Controller
          control={form.control}
          name="prompt"
          rules={{
            validate: (value) => value.trim().length > 0 || "Enter a message.",
          }}
          render={({ field, fieldState }) => (
            <Field className="flex-1 gap-1" data-invalid={fieldState.invalid}>
              <FieldLabel className="sr-only" htmlFor={field.name}>
                Message
              </FieldLabel>
              <Input
                {...field}
                aria-invalid={fieldState.invalid}
                autoComplete="off"
                id={field.name}
                onBlur={(event) => {
                  field.onBlur();
                  window.setTimeout(() => setCursorIndex(-1), 0);
                }}
                onChange={(event) => {
                  field.onChange(event);
                  updateCursorFromInput(event.currentTarget);
                }}
                onClick={(event) => updateCursorFromInput(event.currentTarget)}
                onFocus={(event) => updateCursorFromInput(event.currentTarget)}
                onKeyDown={onPromptKeyDown}
                onKeyUp={(event) => updateCursorFromInput(event.currentTarget)}
                placeholder="Message"
                ref={(element) => {
                  field.ref(element);
                  inputRef.current = element;
                }}
              />
              <FieldError errors={[fieldState.error]} />
            </Field>
          )}
        />
        <Button
          aria-label="Send message"
          disabled={!prompt}
          size="icon"
          type="submit"
        >
          <SendHorizontalIcon />
        </Button>
      </div>
    </form>
  );
}

function findActiveMention(
  value: string,
  cursorIndex: number,
): ActiveMention | null {
  if (cursorIndex < 0) {
    return null;
  }

  const safeCursorIndex = Math.min(cursorIndex, value.length);
  const markerIndex = value.slice(0, safeCursorIndex).lastIndexOf("@");

  if (markerIndex < 0 || !hasMentionPrefixBoundary(value, markerIndex)) {
    return null;
  }

  const query = value.slice(markerIndex + 1, safeCursorIndex);

  if (!isMentionQuery(query)) {
    return null;
  }

  let endIndex = safeCursorIndex;

  while (endIndex < value.length && isMentionCharacter(value[endIndex])) {
    endIndex += 1;
  }

  return {
    markerIndex,
    endIndex,
    query,
  };
}

function mentionOptionMatches(option: SessionMentionOption, query: string) {
  const normalizedQuery = query.toLowerCase();

  if (!normalizedQuery) {
    return true;
  }

  return [option.sessionId, option.name, option.kind].some((value) =>
    value.toLowerCase().includes(normalizedQuery),
  );
}

function hasMentionPrefixBoundary(value: string, markerIndex: number) {
  const previousCharacter = value[markerIndex - 1];

  return !previousCharacter || !isMentionCharacter(previousCharacter);
}

function isMentionQuery(value: string) {
  return [...value].every(isMentionCharacter);
}

function isMentionCharacter(character: string) {
  return /^[a-z0-9_-]$/i.test(character);
}
