import { z } from "zod";

import { ApprovalAnswerSchema } from "./websocketEvents";

export const WebsocketClientMessageType = {
  TurnStart: "turn.start",
  ApprovalRespond: "approval.respond",
} as const;

export type WebsocketClientMessageType =
  (typeof WebsocketClientMessageType)[keyof typeof WebsocketClientMessageType];

export const ApiWebsocketTurnStartMessageSchema = z.object({
  type: z.literal(WebsocketClientMessageType.TurnStart),
  session_id: z.string().nullish(),
  prompt: z.string(),
});

export const ApiWebsocketApprovalRespondMessageSchema = z.object({
  type: z.literal(WebsocketClientMessageType.ApprovalRespond),
  session_id: z.string(),
  answer: ApprovalAnswerSchema,
});

export const ApiWebsocketClientMessageSchema = z.discriminatedUnion("type", [
  ApiWebsocketTurnStartMessageSchema,
  ApiWebsocketApprovalRespondMessageSchema,
]);

export type ApiWebsocketTurnStartMessage = z.infer<
  typeof ApiWebsocketTurnStartMessageSchema
>;
export type ApiWebsocketApprovalRespondMessage = z.infer<
  typeof ApiWebsocketApprovalRespondMessageSchema
>;
export type ApiWebsocketClientMessage = z.infer<
  typeof ApiWebsocketClientMessageSchema
>;

export const WebsocketTurnStartMessageSchema = z.object({
  type: z.literal(WebsocketClientMessageType.TurnStart),
  sessionId: z.string().nullish(),
  prompt: z.string(),
});

export const WebsocketApprovalRespondMessageSchema = z.object({
  type: z.literal(WebsocketClientMessageType.ApprovalRespond),
  sessionId: z.string(),
  answer: ApprovalAnswerSchema,
});

export const WebsocketClientMessageInputSchema = z.discriminatedUnion("type", [
  WebsocketTurnStartMessageSchema,
  WebsocketApprovalRespondMessageSchema,
]);

export type WebsocketTurnStartMessage = z.infer<
  typeof WebsocketTurnStartMessageSchema
>;
export type WebsocketApprovalRespondMessage = z.infer<
  typeof WebsocketApprovalRespondMessageSchema
>;

export type WebsocketClientMessage =
  | WebsocketTurnStartMessage
  | WebsocketApprovalRespondMessage;

export const WebsocketClientMessageSchema =
  WebsocketClientMessageInputSchema.transform((message): ApiWebsocketClientMessage => {
    switch (message.type) {
      case WebsocketClientMessageType.TurnStart:
        return {
          type: message.type,
          session_id: message.sessionId,
          prompt: message.prompt,
        };
      case WebsocketClientMessageType.ApprovalRespond:
        return {
          type: message.type,
          session_id: message.sessionId,
          answer: message.answer,
        };
    }
  });
