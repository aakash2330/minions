import { z } from "zod";

export enum ApiWorkspaceChatMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export enum ApiWorkspaceChatMessageStatus {
  Pending = "pending",
  Streaming = "streaming",
  Complete = "complete",
  Error = "error",
}

export const ApiWorkspaceChatMessageSchema = z
  .object({
    id: z.string(),
    workspace_id: z.string(),
    session_id: z.string().nullable(),
    session_message_id: z.string().nullable(),
    parent_message_id: z.string().nullable(),
    role: z.enum(ApiWorkspaceChatMessageRole),
    text: z.string(),
    status: z.enum(ApiWorkspaceChatMessageStatus),
  })
  .transform(
    ({
      workspace_id,
      session_id,
      session_message_id,
      parent_message_id,
      ...message
    }) => ({
      ...message,
      workspaceId: workspace_id,
      sessionId: session_id,
      sessionMessageId: session_message_id,
      parentMessageId: parent_message_id,
    }),
  );

export const ApiWorkspaceChatMessagesResponseSchema = z.array(
  ApiWorkspaceChatMessageSchema,
);

export type ApiWorkspaceChatMessage = z.infer<
  typeof ApiWorkspaceChatMessageSchema
>;
