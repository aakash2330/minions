import { z } from "zod";

import { ApprovalAnswerSchema } from "./serverEvent";

export const ServerMessageType = {
  TurnStart: "turn.start",
  ApprovalResponse: "approval.respond",
} as const;

export type ServerMessageType =
  (typeof ServerMessageType)[keyof typeof ServerMessageType];

export const ServerTurnStartMessagePayloadSchema = z.object({
  type: z.literal(ServerMessageType.TurnStart),
  session_id: z.string().nullish(),
  prompt: z.string(),
});

export const ServerApprovalResponseMessagePayloadSchema = z.object({
  type: z.literal(ServerMessageType.ApprovalResponse),
  session_id: z.string(),
  answer: ApprovalAnswerSchema,
});

export const ServerMessagePayloadSchema = z.discriminatedUnion("type", [
  ServerTurnStartMessagePayloadSchema,
  ServerApprovalResponseMessagePayloadSchema,
]);

export type ServerTurnStartMessagePayload = z.infer<
  typeof ServerTurnStartMessagePayloadSchema
>;
export type ServerApprovalResponseMessagePayload = z.infer<
  typeof ServerApprovalResponseMessagePayloadSchema
>;
export type ServerMessagePayload = z.infer<typeof ServerMessagePayloadSchema>;

export const ServerTurnStartMessageSchema = z.object({
  type: z.literal(ServerMessageType.TurnStart),
  sessionId: z.string().min(1),
  prompt: z.string(),
});

export const ServerApprovalResponseMessageSchema = z.object({
  type: z.literal(ServerMessageType.ApprovalResponse),
  sessionId: z.string().min(1),
  answer: ApprovalAnswerSchema,
});

export const ServerMessageInputSchema = z.discriminatedUnion("type", [
  ServerTurnStartMessageSchema,
  ServerApprovalResponseMessageSchema,
]);

export type ServerTurnStartMessage = z.infer<
  typeof ServerTurnStartMessageSchema
>;
export type ServerApprovalResponseMessage = z.infer<
  typeof ServerApprovalResponseMessageSchema
>;
export type ServerMessage =
  | ServerTurnStartMessage
  | ServerApprovalResponseMessage;

export const ServerMessageSchema = ServerMessageInputSchema.transform(
  (message): ServerMessagePayload => {
    switch (message.type) {
      case ServerMessageType.TurnStart:
        return {
          type: message.type,
          session_id: message.sessionId,
          prompt: message.prompt,
        };
      case ServerMessageType.ApprovalResponse:
        return {
          type: message.type,
          session_id: message.sessionId,
          answer: message.answer,
        };
    }
  },
);
