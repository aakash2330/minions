import { z } from "zod";

import { ApprovalAnswerSchema } from "../events/wsEvent";

export const WsMessageType = {
  TurnStart: "turn.start",
  WorkspaceChatTurnStart: "workspace_chat.turn.start",
  ApprovalResponse: "approval.respond",
} as const;

export type WsMessageType = (typeof WsMessageType)[keyof typeof WsMessageType];

export const WsTurnStartMessagePayloadSchema = z.object({
  type: z.literal(WsMessageType.TurnStart),
  session_id: z.string().nullish(),
  prompt: z.string(),
});

export const WsApprovalResponseMessagePayloadSchema = z.object({
  type: z.literal(WsMessageType.ApprovalResponse),
  session_id: z.string(),
  answer: ApprovalAnswerSchema,
});

export const WsWorkspaceChatTurnStartMessagePayloadSchema = z.object({
  type: z.literal(WsMessageType.WorkspaceChatTurnStart),
  workspace_id: z.string(),
  prompt: z.string(),
});

export const WsMessagePayloadSchema = z.discriminatedUnion("type", [
  WsTurnStartMessagePayloadSchema,
  WsWorkspaceChatTurnStartMessagePayloadSchema,
  WsApprovalResponseMessagePayloadSchema,
]);

export type WsTurnStartMessagePayload = z.infer<
  typeof WsTurnStartMessagePayloadSchema
>;
export type WsApprovalResponseMessagePayload = z.infer<
  typeof WsApprovalResponseMessagePayloadSchema
>;
export type WsWorkspaceChatTurnStartMessagePayload = z.infer<
  typeof WsWorkspaceChatTurnStartMessagePayloadSchema
>;
export type WsMessagePayload = z.infer<typeof WsMessagePayloadSchema>;

export const WsTurnStartMessageSchema = z.object({
  type: z.literal(WsMessageType.TurnStart),
  sessionId: z.string().min(1),
  prompt: z.string(),
});

export const WsApprovalResponseMessageSchema = z.object({
  type: z.literal(WsMessageType.ApprovalResponse),
  sessionId: z.string().min(1),
  answer: ApprovalAnswerSchema,
});

export const WsWorkspaceChatTurnStartMessageSchema = z.object({
  type: z.literal(WsMessageType.WorkspaceChatTurnStart),
  workspaceId: z.string().min(1),
  prompt: z.string(),
});

export const WsMessageInputSchema = z.discriminatedUnion("type", [
  WsTurnStartMessageSchema,
  WsWorkspaceChatTurnStartMessageSchema,
  WsApprovalResponseMessageSchema,
]);

export type WsTurnStartMessage = z.infer<typeof WsTurnStartMessageSchema>;
export type WsApprovalResponseMessage = z.infer<
  typeof WsApprovalResponseMessageSchema
>;
export type WsWorkspaceChatTurnStartMessage = z.infer<
  typeof WsWorkspaceChatTurnStartMessageSchema
>;
export type WsMessage =
  | WsTurnStartMessage
  | WsWorkspaceChatTurnStartMessage
  | WsApprovalResponseMessage;

export const WsMessageSchema = WsMessageInputSchema.transform(
  (message): WsMessagePayload => {
    switch (message.type) {
      case WsMessageType.TurnStart:
        return {
          type: message.type,
          session_id: message.sessionId,
          prompt: message.prompt,
        };
      case WsMessageType.WorkspaceChatTurnStart:
        return {
          type: message.type,
          workspace_id: message.workspaceId,
          prompt: message.prompt,
        };
      case WsMessageType.ApprovalResponse:
        return {
          type: message.type,
          session_id: message.sessionId,
          answer: message.answer,
        };
    }
  },
);
