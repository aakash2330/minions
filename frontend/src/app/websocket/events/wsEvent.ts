import { z } from "zod";

import { SessionInteractionType } from "@/game/sessionInteractions";

export const WsEventType = {
  TurnStarted: "turn.started",
  AssistantDelta: "assistant.delta",
  TurnCompleted: "turn.completed",
  ApprovalRequest: "approval.request",
  ApprovalResolved: "approval.resolved",
  WorkspaceChatMessageCreated: "workspace_chat.message.created",
  WorkspaceChatMessageDelta: "workspace_chat.message.delta",
  WorkspaceChatMessageCompleted: "workspace_chat.message.completed",
  SessionInteraction: "session.interaction",
  Error: "error",
} as const;

export type WsEventType = (typeof WsEventType)[keyof typeof WsEventType];

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

export const WsTurnStartedEventPayloadSchema = z.object({
  type: z.literal(WsEventType.TurnStarted),
  session_id: z.string(),
});

export const WsAssistantDeltaEventPayloadSchema = z.object({
  type: z.literal(WsEventType.AssistantDelta),
  session_id: z.string(),
  message_id: z.string(),
  text: z.string(),
});

export const WsTurnCompletedEventPayloadSchema = z.object({
  type: z.literal(WsEventType.TurnCompleted),
  session_id: z.string(),
});

export const WsApprovalRequestEventPayloadSchema = z.object({
  type: z.literal(WsEventType.ApprovalRequest),
  session_id: z.string(),
  workspace_id: z.string().optional(),
  method: z.string(),
  params: z.unknown(),
  question: z.string(),
  answers: z.array(ApprovalAnswerSchema),
});

export const WsApprovalResolvedEventPayloadSchema = z.object({
  type: z.literal(WsEventType.ApprovalResolved),
  session_id: z.string(),
  workspace_id: z.string().optional(),
});

export const WsWorkspaceChatMessageCreatedEventPayloadSchema = z.object({
  type: z.literal(WsEventType.WorkspaceChatMessageCreated),
  workspace_id: z.string(),
  message_id: z.string(),
  session_id: z.string().nullable(),
  role: z.string(),
  text: z.string(),
  status: z.string(),
});

export const WsWorkspaceChatMessageDeltaEventPayloadSchema = z.object({
  type: z.literal(WsEventType.WorkspaceChatMessageDelta),
  workspace_id: z.string(),
  message_id: z.string(),
  session_id: z.string().nullable(),
  text: z.string(),
});

export const WsWorkspaceChatMessageCompletedEventPayloadSchema = z.object({
  type: z.literal(WsEventType.WorkspaceChatMessageCompleted),
  workspace_id: z.string(),
  message_id: z.string(),
  session_id: z.string().nullable(),
  status: z.string(),
  text: z.string().optional(),
});

export const WsSessionInteractionEventPayloadSchema = z.object({
  type: z.literal(WsEventType.SessionInteraction),
  session_id: z.string(),
  interaction_type: z.enum(SessionInteractionType),
});

export const WsErrorEventPayloadSchema = z.object({
  type: z.literal(WsEventType.Error),
  session_id: z.string().optional(),
  message: z.string(),
});

export const WsEventPayloadSchema = z.discriminatedUnion("type", [
  WsTurnStartedEventPayloadSchema,
  WsAssistantDeltaEventPayloadSchema,
  WsTurnCompletedEventPayloadSchema,
  WsApprovalRequestEventPayloadSchema,
  WsApprovalResolvedEventPayloadSchema,
  WsWorkspaceChatMessageCreatedEventPayloadSchema,
  WsWorkspaceChatMessageDeltaEventPayloadSchema,
  WsWorkspaceChatMessageCompletedEventPayloadSchema,
  WsSessionInteractionEventPayloadSchema,
  WsErrorEventPayloadSchema,
]);

export type WsTurnStartedEventPayload = z.infer<
  typeof WsTurnStartedEventPayloadSchema
>;
export type WsAssistantDeltaEventPayload = z.infer<
  typeof WsAssistantDeltaEventPayloadSchema
>;
export type WsTurnCompletedEventPayload = z.infer<
  typeof WsTurnCompletedEventPayloadSchema
>;
export type WsApprovalRequestEventPayload = z.infer<
  typeof WsApprovalRequestEventPayloadSchema
>;
export type WsApprovalResolvedEventPayload = z.infer<
  typeof WsApprovalResolvedEventPayloadSchema
>;
export type WsWorkspaceChatMessageCreatedEventPayload = z.infer<
  typeof WsWorkspaceChatMessageCreatedEventPayloadSchema
>;
export type WsWorkspaceChatMessageDeltaEventPayload = z.infer<
  typeof WsWorkspaceChatMessageDeltaEventPayloadSchema
>;
export type WsWorkspaceChatMessageCompletedEventPayload = z.infer<
  typeof WsWorkspaceChatMessageCompletedEventPayloadSchema
>;
export type WsSessionInteractionEventPayload = z.infer<
  typeof WsSessionInteractionEventPayloadSchema
>;
export type WsErrorEventPayload = z.infer<typeof WsErrorEventPayloadSchema>;
export type WsEventPayload = z.infer<typeof WsEventPayloadSchema>;

export type WsTurnStartedEvent = {
  type: typeof WsEventType.TurnStarted;
  sessionId: string;
};

export type WsAssistantDeltaEvent = {
  type: typeof WsEventType.AssistantDelta;
  sessionId: string;
  messageId: string;
  text: string;
};

export type WsTurnCompletedEvent = {
  type: typeof WsEventType.TurnCompleted;
  sessionId: string;
};

export type WsApprovalRequestEvent = {
  type: typeof WsEventType.ApprovalRequest;
  sessionId: string;
  workspaceId?: string;
  method: string;
  params: unknown;
  question: string;
  answers: ApprovalAnswer[];
};

export type WsApprovalResolvedEvent = {
  type: typeof WsEventType.ApprovalResolved;
  sessionId: string;
  workspaceId?: string;
};

export type WsWorkspaceChatMessageCreatedEvent = {
  type: typeof WsEventType.WorkspaceChatMessageCreated;
  workspaceId: string;
  messageId: string;
  sessionId: string | null;
  role: string;
  text: string;
  status: string;
};

export type WsWorkspaceChatMessageDeltaEvent = {
  type: typeof WsEventType.WorkspaceChatMessageDelta;
  workspaceId: string;
  messageId: string;
  sessionId: string | null;
  text: string;
};

export type WsWorkspaceChatMessageCompletedEvent = {
  type: typeof WsEventType.WorkspaceChatMessageCompleted;
  workspaceId: string;
  messageId: string;
  sessionId: string | null;
  status: string;
  text?: string;
};

export type WsSessionInteractionEvent = {
  type: typeof WsEventType.SessionInteraction;
  sessionId: string;
  interactionType: SessionInteractionType;
};

export type WsErrorEvent = {
  type: typeof WsEventType.Error;
  sessionId?: string;
  message: string;
};

export type WsEvent =
  | WsTurnStartedEvent
  | WsAssistantDeltaEvent
  | WsTurnCompletedEvent
  | WsApprovalRequestEvent
  | WsApprovalResolvedEvent
  | WsWorkspaceChatMessageCreatedEvent
  | WsWorkspaceChatMessageDeltaEvent
  | WsWorkspaceChatMessageCompletedEvent
  | WsSessionInteractionEvent
  | WsErrorEvent;

export const WsEventSchema = WsEventPayloadSchema.transform(
  (event): WsEvent => {
    switch (event.type) {
      case WsEventType.TurnStarted:
        return {
          type: event.type,
          sessionId: event.session_id,
        };
      case WsEventType.AssistantDelta:
        return {
          type: event.type,
          sessionId: event.session_id,
          messageId: event.message_id,
          text: event.text,
        };
      case WsEventType.TurnCompleted:
        return {
          type: event.type,
          sessionId: event.session_id,
        };
      case WsEventType.ApprovalRequest:
        return {
          type: event.type,
          sessionId: event.session_id,
          ...(event.workspace_id === undefined
            ? {}
            : { workspaceId: event.workspace_id }),
          method: event.method,
          params: event.params,
          question: event.question,
          answers: event.answers,
        };
      case WsEventType.ApprovalResolved:
        return {
          type: event.type,
          sessionId: event.session_id,
          ...(event.workspace_id === undefined
            ? {}
            : { workspaceId: event.workspace_id }),
        };
      case WsEventType.WorkspaceChatMessageCreated:
        return {
          type: event.type,
          workspaceId: event.workspace_id,
          messageId: event.message_id,
          sessionId: event.session_id,
          role: event.role,
          text: event.text,
          status: event.status,
        };
      case WsEventType.WorkspaceChatMessageDelta:
        return {
          type: event.type,
          workspaceId: event.workspace_id,
          messageId: event.message_id,
          sessionId: event.session_id,
          text: event.text,
        };
      case WsEventType.WorkspaceChatMessageCompleted:
        return {
          type: event.type,
          workspaceId: event.workspace_id,
          messageId: event.message_id,
          sessionId: event.session_id,
          status: event.status,
          ...(event.text === undefined ? {} : { text: event.text }),
        };
      case WsEventType.SessionInteraction:
        return {
          type: event.type,
          sessionId: event.session_id,
          interactionType: event.interaction_type,
        };
      case WsEventType.Error:
        return {
          type: event.type,
          message: event.message,
          ...(event.session_id === undefined
            ? {}
            : { sessionId: event.session_id }),
        };
    }
  },
);
