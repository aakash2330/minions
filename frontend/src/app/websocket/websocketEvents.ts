import { z } from "zod";

export const WebsocketServerEventType = {
  TurnStarted: "turn.started",
  AssistantDelta: "assistant.delta",
  TurnCompleted: "turn.completed",
  ApprovalRequest: "approval.request",
  Error: "error",
} as const;

export type WebsocketServerEventType =
  (typeof WebsocketServerEventType)[keyof typeof WebsocketServerEventType];

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

export const ApiWebsocketTurnStartedEventSchema = z.object({
  type: z.literal(WebsocketServerEventType.TurnStarted),
  session_id: z.string(),
});

export const ApiWebsocketAssistantDeltaEventSchema = z.object({
  type: z.literal(WebsocketServerEventType.AssistantDelta),
  session_id: z.string(),
  text: z.string(),
});

export const ApiWebsocketTurnCompletedEventSchema = z.object({
  type: z.literal(WebsocketServerEventType.TurnCompleted),
  session_id: z.string(),
});

export const ApiWebsocketApprovalRequestEventSchema = z.object({
  type: z.literal(WebsocketServerEventType.ApprovalRequest),
  session_id: z.string(),
  method: z.string(),
  params: z.unknown(),
  question: z.string(),
  answers: z.array(ApprovalAnswerSchema),
});

export const ApiWebsocketErrorEventSchema = z.object({
  type: z.literal(WebsocketServerEventType.Error),
  session_id: z.string().optional(),
  message: z.string(),
});

export const ApiWebsocketServerEventSchema = z.discriminatedUnion("type", [
  ApiWebsocketTurnStartedEventSchema,
  ApiWebsocketAssistantDeltaEventSchema,
  ApiWebsocketTurnCompletedEventSchema,
  ApiWebsocketApprovalRequestEventSchema,
  ApiWebsocketErrorEventSchema,
]);

export type ApiWebsocketTurnStartedEvent = z.infer<
  typeof ApiWebsocketTurnStartedEventSchema
>;
export type ApiWebsocketAssistantDeltaEvent = z.infer<
  typeof ApiWebsocketAssistantDeltaEventSchema
>;
export type ApiWebsocketTurnCompletedEvent = z.infer<
  typeof ApiWebsocketTurnCompletedEventSchema
>;
export type ApiWebsocketApprovalRequestEvent = z.infer<
  typeof ApiWebsocketApprovalRequestEventSchema
>;
export type ApiWebsocketErrorEvent = z.infer<
  typeof ApiWebsocketErrorEventSchema
>;
export type ApiWebsocketServerEvent = z.infer<
  typeof ApiWebsocketServerEventSchema
>;

export type WebsocketTurnStartedEvent = {
  type: typeof WebsocketServerEventType.TurnStarted;
  sessionId: string;
};

export type WebsocketAssistantDeltaEvent = {
  type: typeof WebsocketServerEventType.AssistantDelta;
  sessionId: string;
  text: string;
};

export type WebsocketTurnCompletedEvent = {
  type: typeof WebsocketServerEventType.TurnCompleted;
  sessionId: string;
};

export type WebsocketApprovalRequestEvent = {
  type: typeof WebsocketServerEventType.ApprovalRequest;
  sessionId: string;
  method: string;
  params: unknown;
  question: string;
  answers: ApprovalAnswer[];
};

export type WebsocketErrorEvent = {
  type: typeof WebsocketServerEventType.Error;
  sessionId?: string;
  message: string;
};

export type WebsocketServerEvent =
  | WebsocketTurnStartedEvent
  | WebsocketAssistantDeltaEvent
  | WebsocketTurnCompletedEvent
  | WebsocketApprovalRequestEvent
  | WebsocketErrorEvent;

export const WebsocketServerEventSchema =
  ApiWebsocketServerEventSchema.transform((event): WebsocketServerEvent => {
    switch (event.type) {
      case WebsocketServerEventType.TurnStarted:
        return {
          type: event.type,
          sessionId: event.session_id,
        };
      case WebsocketServerEventType.AssistantDelta:
        return {
          type: event.type,
          sessionId: event.session_id,
          text: event.text,
        };
      case WebsocketServerEventType.TurnCompleted:
        return {
          type: event.type,
          sessionId: event.session_id,
        };
      case WebsocketServerEventType.ApprovalRequest:
        return {
          type: event.type,
          sessionId: event.session_id,
          method: event.method,
          params: event.params,
          question: event.question,
          answers: event.answers,
        };
      case WebsocketServerEventType.Error:
        return {
          type: event.type,
          message: event.message,
          ...(event.session_id === undefined
            ? {}
            : { sessionId: event.session_id }),
        };
    }
  });
