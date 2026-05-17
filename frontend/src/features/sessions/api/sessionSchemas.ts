import { z } from "zod";

import { Direction } from "@/game/characters/characterConfig";

export enum ApiSessionMessageStatus {
  Pending = "pending",
  Streaming = "streaming",
  Complete = "complete",
  Error = "error",
}

export const ApiPointSchema = z.object({
  x: z.number(),
  y: z.number(),
});

export const ApiDirectionSchema = z.enum(Direction);

export const ApiPointWithFacingSchema = ApiPointSchema.extend({
  facing: ApiDirectionSchema,
});

export const ApiSessionElementSchema = z
  .object({
    id: z.string(),
    assignedSessionId: z.string().nullable(),
    kind: z.string(),
    label: z.string(),
    position: ApiPointSchema,
    facing: ApiDirectionSchema,
  })
  .transform(({ assignedSessionId, ...element }) => ({
    ...element,
    assignedSessionId,
  }));

export const ApiSessionMessageSchema = z
  .object({
    id: z.string(),
    session_id: z.string(),
    role: z.string(),
    text: z.string(),
    status: z.enum(ApiSessionMessageStatus),
  })
  .transform(({ session_id, ...message }) => ({
    ...message,
    sessionId: session_id,
  }));

export const ApiSessionSchema = z
  .object({
    session_id: z.string(),
    workspaceId: z.string(),
    name: z.string(),
    kind: z.string(),
    status: z.string(),
    spawn: ApiPointWithFacingSchema,
    current: ApiPointWithFacingSchema,
    messages: z.array(ApiSessionMessageSchema),
  })
  .transform(({ session_id, ...session }) => ({
    ...session,
    sessionId: session_id,
  }));

export const ApiWorkspaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  rootPath: z.string().nullable(),
});

export const ApiSessionsResponseSchema = z.array(ApiSessionSchema);
export const ApiWorkspaceElementsSchema = z.array(ApiSessionElementSchema);
export const ApiWorkspaceResponseSchema = ApiWorkspaceSchema;
export const ApiWorkspacesResponseSchema = z.array(ApiWorkspaceSchema);

export type ApiPoint = z.infer<typeof ApiPointSchema>;
export type ApiPointWithFacing = z.infer<typeof ApiPointWithFacingSchema>;
export type ApiSessionElement = z.infer<typeof ApiSessionElementSchema>;
export type ApiSession = z.infer<typeof ApiSessionSchema>;
export type ApiSessionMessage = z.infer<typeof ApiSessionMessageSchema>;
export type ApiWorkspace = z.infer<typeof ApiWorkspaceSchema>;
