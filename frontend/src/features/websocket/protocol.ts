import { z } from "zod";

export enum ClientMessageType {
  ApprovalRespond = "approval.respond",
  SessionStart = "session.start",
  TurnStart = "turn.start",
}

export enum ServerEventType {
  ApprovalRequest = "approval.request",
  AssistantDelta = "assistant.delta",
  Error = "error",
  SessionReady = "session.ready",
  TurnCompleted = "turn.completed",
  TurnStarted = "turn.started",
}

export enum ApprovalAnswer {
  Accept = "accept",
  AcceptForSession = "acceptForSession",
  Cancel = "cancel",
  Decline = "decline",
}

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

const JsonValueSchema: z.ZodType<JsonValue> = z.lazy(() =>
  z.union([
    z.null(),
    z.boolean(),
    z.number(),
    z.string(),
    z.array(JsonValueSchema),
    z.record(z.string(), JsonValueSchema),
  ]),
);

const ApprovalAnswerSchema = z.enum([
  ApprovalAnswer.Accept,
  ApprovalAnswer.AcceptForSession,
  ApprovalAnswer.Cancel,
  ApprovalAnswer.Decline,
]);

const ClientMessageSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal(ClientMessageType.ApprovalRespond),
    answer: ApprovalAnswerSchema,
    sessionId: z.string(),
  }),
  z.object({
    type: z.literal(ClientMessageType.SessionStart),
    cwd: z.string(),
    sessionId: z.string(),
  }),
  z.object({
    type: z.literal(ClientMessageType.TurnStart),
    prompt: z.string(),
    sessionId: z.string(),
  }),
]);

const ClientMessageRequestTransformSchema = ClientMessageSchema.transform(
  (message) => {
    switch (message.type) {
      case ClientMessageType.ApprovalRespond:
        return {
          type: message.type,
          answer: message.answer,
          session_id: message.sessionId,
        };
      case ClientMessageType.SessionStart:
        return {
          type: message.type,
          cwd: message.cwd,
          session_id: message.sessionId,
        };
      case ClientMessageType.TurnStart:
        return {
          type: message.type,
          prompt: message.prompt,
          session_id: message.sessionId,
        };
      default:
        return assertNever(message);
    }
  },
);

const ServerEventResponseSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal(ServerEventType.ApprovalRequest),
    answers: z.array(ApprovalAnswerSchema),
    method: z.string(),
    params: JsonValueSchema,
    question: z.string(),
    session_id: z.string(),
  }),
  z.object({
    type: z.literal(ServerEventType.AssistantDelta),
    session_id: z.string(),
    text: z.string(),
  }),
  z.object({
    type: z.literal(ServerEventType.Error),
    message: z.string(),
    session_id: z.string().nullable().optional(),
  }),
  z.object({
    type: z.literal(ServerEventType.SessionReady),
    session_id: z.string(),
  }),
  z.object({
    type: z.literal(ServerEventType.TurnCompleted),
    session_id: z.string(),
  }),
  z.object({
    type: z.literal(ServerEventType.TurnStarted),
    session_id: z.string(),
  }),
]);

const ServerEventTransformSchema = ServerEventResponseSchema.transform(
  (event): ServerEvent => {
    switch (event.type) {
      case ServerEventType.ApprovalRequest:
        return {
          type: ServerEventType.ApprovalRequest,
          answers: event.answers,
          method: event.method,
          params: event.params,
          question: event.question,
          sessionId: event.session_id,
        };
      case ServerEventType.AssistantDelta:
        return {
          type: ServerEventType.AssistantDelta,
          sessionId: event.session_id,
          text: event.text,
        };
      case ServerEventType.Error:
        return {
          type: ServerEventType.Error,
          message: event.message,
          sessionId: event.session_id ?? null,
        };
      case ServerEventType.SessionReady:
        return {
          type: ServerEventType.SessionReady,
          sessionId: event.session_id,
        };
      case ServerEventType.TurnCompleted:
        return {
          type: ServerEventType.TurnCompleted,
          sessionId: event.session_id,
        };
      case ServerEventType.TurnStarted:
        return {
          type: ServerEventType.TurnStarted,
          sessionId: event.session_id,
        };
      default:
        return assertNever(event);
    }
  },
);

export type ClientMessage = z.infer<typeof ClientMessageSchema>;

export type SessionReadyEvent = {
  type: ServerEventType.SessionReady;
  sessionId: string;
};

export type TurnStartedEvent = {
  type: ServerEventType.TurnStarted;
  sessionId: string;
};

export type AssistantDeltaEvent = {
  type: ServerEventType.AssistantDelta;
  sessionId: string;
  text: string;
};

export type TurnCompletedEvent = {
  type: ServerEventType.TurnCompleted;
  sessionId: string;
};

export type ApprovalRequestEvent = {
  type: ServerEventType.ApprovalRequest;
  answers: ApprovalAnswer[];
  method: string;
  params: JsonValue;
  question: string;
  sessionId: string;
};

export type ErrorEvent = {
  type: ServerEventType.Error;
  message: string;
  sessionId: string | null;
};

export type ServerEvent =
  | ApprovalRequestEvent
  | AssistantDeltaEvent
  | ErrorEvent
  | SessionReadyEvent
  | TurnCompletedEvent
  | TurnStartedEvent;

export type ServerEventByType<TType extends ServerEventType> = Extract<
  ServerEvent,
  { type: TType }
>;

export function parseServerEvent(rawMessage: string): ServerEvent | null {
  try {
    const value = JSON.parse(rawMessage) as unknown;
    const result = ServerEventTransformSchema.safeParse(value);

    if (!result.success) {
      return null;
    }

    return result.data;
  } catch {
    return null;
  }
}

export function serializeClientMessage(message: ClientMessage) {
  return ClientMessageRequestTransformSchema.parse(message);
}

function assertNever(value: never): never {
  throw new Error(`Unhandled server event: ${JSON.stringify(value)}`);
}
