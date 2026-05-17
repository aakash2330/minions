import { Button } from "@/components/ui/button";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { CheckIcon, SendHorizontalIcon, XIcon } from "lucide-react";
import { Controller, useForm } from "react-hook-form";

type SessionMessageSubmitButtonProps = {
  isApprovalRequestPending: boolean;
  isApprovalResponsePending: boolean;
  onAccept: () => void;
  onDecline: () => void;
  onPromptSubmit: ({ prompt }: { prompt: string }) => void;
};

type SessionMessageFormValues = {
  prompt: string;
};

export function SessionMessageSubmitButton({
  isApprovalRequestPending,
  isApprovalResponsePending,
  onPromptSubmit,
  onAccept,
  onDecline,
}: SessionMessageSubmitButtonProps) {
  const form = useForm<SessionMessageFormValues>({
    defaultValues: {
      prompt: "",
    },
  });
  const prompt = form.watch("prompt").trim();

  function onSubmit(values: SessionMessageFormValues) {
    const prompt = values.prompt.trim();
    if (!prompt) return;
    onPromptSubmit({ prompt });
    form.reset();
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
      className="flex shrink-0 items-start gap-2 border-t border-border bg-popover p-4"
      onSubmit={form.handleSubmit(onSubmit)}
    >
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
              placeholder="Message"
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
    </form>
  );
}
