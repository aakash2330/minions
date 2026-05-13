import { z } from "zod";

export const ServerEventType = {
  TurnStarted: "turn.started",
  AssistantDelta: "assistant.delta",
  TurnCompleted: "turn.completed",
  ApprovalRequest: "approval.request",
  Error: "error",
} as const;

export type ServerEventType =
  (typeof ServerEventType)[keyof typeof ServerEventType];

export const ApprovalAnswer = {
  Accept: "accept",
  AcceptForSession: "acceptForSession",
  Cancel: "cancel",
  Decline: "decline",
} as const;

export const ApprovalAnswerSchema = z.enum([
  ApprovalAnswer.Accept,
  ApprovalAnswer.AcceptForSession,
  ApprovalAnswer.Cancel,
  ApprovalAnswer.Decline,
]);

export type ApprovalAnswer =
  (typeof ApprovalAnswer)[keyof typeof ApprovalAnswer];

export const ServerTurnStartedEventPayloadSchema = z.object({
  type: z.literal(ServerEventType.TurnStarted),
  session_id: z.string(),
});

export const ServerAssistantDeltaEventPayloadSchema = z.object({
  type: z.literal(ServerEventType.AssistantDelta),
  session_id: z.string(),
  text: z.string(),
});

export const ServerTurnCompletedEventPayloadSchema = z.object({
  type: z.literal(ServerEventType.TurnCompleted),
  session_id: z.string(),
});

export const ServerApprovalRequestEventPayloadSchema = z.object({
  type: z.literal(ServerEventType.ApprovalRequest),
  session_id: z.string(),
  method: z.string(),
  params: z.unknown(),
  question: z.string(),
  answers: z.array(ApprovalAnswerSchema),
});

export const ServerErrorEventPayloadSchema = z.object({
  type: z.literal(ServerEventType.Error),
  session_id: z.string().optional(),
  message: z.string(),
});

export const ServerEventPayloadSchema = z.discriminatedUnion("type", [
  ServerTurnStartedEventPayloadSchema,
  ServerAssistantDeltaEventPayloadSchema,
  ServerTurnCompletedEventPayloadSchema,
  ServerApprovalRequestEventPayloadSchema,
  ServerErrorEventPayloadSchema,
]);

export type ServerTurnStartedEventPayload = z.infer<
  typeof ServerTurnStartedEventPayloadSchema
>;
export type ServerAssistantDeltaEventPayload = z.infer<
  typeof ServerAssistantDeltaEventPayloadSchema
>;
export type ServerTurnCompletedEventPayload = z.infer<
  typeof ServerTurnCompletedEventPayloadSchema
>;
export type ServerApprovalRequestEventPayload = z.infer<
  typeof ServerApprovalRequestEventPayloadSchema
>;
export type ServerErrorEventPayload = z.infer<
  typeof ServerErrorEventPayloadSchema
>;
export type ServerEventPayload = z.infer<typeof ServerEventPayloadSchema>;

export type ServerTurnStartedEvent = {
  type: typeof ServerEventType.TurnStarted;
  sessionId: string;
};

export type ServerAssistantDeltaEvent = {
  type: typeof ServerEventType.AssistantDelta;
  sessionId: string;
  text: string;
};

export type ServerTurnCompletedEvent = {
  type: typeof ServerEventType.TurnCompleted;
  sessionId: string;
};

export type ServerApprovalRequestEvent = {
  type: typeof ServerEventType.ApprovalRequest;
  sessionId: string;
  method: string;
  params: unknown;
  question: string;
  answers: ApprovalAnswer[];
};

export type ServerErrorEvent = {
  type: typeof ServerEventType.Error;
  sessionId?: string;
  message: string;
};

export type ServerEvent =
  | ServerTurnStartedEvent
  | ServerAssistantDeltaEvent
  | ServerTurnCompletedEvent
  | ServerApprovalRequestEvent
  | ServerErrorEvent;

export const ServerEventSchema = ServerEventPayloadSchema.transform((event): ServerEvent => {
  switch (event.type) {
    case ServerEventType.TurnStarted:
      return {
        type: event.type,
        sessionId: event.session_id,
      };
    case ServerEventType.AssistantDelta:
      return {
        type: event.type,
        sessionId: event.session_id,
        text: event.text,
      };
    case ServerEventType.TurnCompleted:
      return {
        type: event.type,
        sessionId: event.session_id,
      };
    case ServerEventType.ApprovalRequest:
      return {
        type: event.type,
        sessionId: event.session_id,
        method: event.method,
        params: event.params,
        question: event.question,
        answers: event.answers,
      };
    case ServerEventType.Error:
      return {
        type: event.type,
        message: event.message,
        ...(event.session_id === undefined
          ? {}
          : { sessionId: event.session_id }),
      };
  }
});
