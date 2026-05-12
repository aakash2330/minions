import { z } from "zod";

export const ApiPointSchema = z.object({
  x: z.number(),
  y: z.number(),
});

export const ApiPointWithFacingSchema = ApiPointSchema.extend({
  facing: z.string(),
});

export const ApiMinionElementSchema = z
  .object({
    id: z.string(),
    assignedSessionId: z.string().nullable(),
    kind: z.string(),
    label: z.string(),
    position: ApiPointSchema,
    facing: z.string(),
  })
  .transform(({ assignedSessionId, ...element }) => ({
    ...element,
    assignedMinionId: assignedSessionId,
  }));

export const ApiMinionMessageSchema = z
  .object({
    id: z.string(),
    session_id: z.string(),
    role: z.string(),
    text: z.string(),
    status: z.string(),
  })
  .transform(({ session_id, ...message }) => ({
    ...message,
    minionId: session_id,
  }));

export const ApiMinionSchema = z
  .object({
    session_id: z.string(),
    workspaceId: z.string(),
    name: z.string(),
    kind: z.string(),
    status: z.string(),
    spawn: ApiPointWithFacingSchema,
    current: ApiPointWithFacingSchema,
    messages: z.array(ApiMinionMessageSchema),
  })
  .transform(({ session_id, ...minion }) => ({
    ...minion,
    minionId: session_id,
  }));

export const ApiWorkspaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  rootPath: z.string().nullable(),
});

export const ApiMinionsResponseSchema = z.array(ApiMinionSchema);
export const ApiWorkspaceElementsSchema = z.array(ApiMinionElementSchema);
export const ApiWorkspaceResponseSchema = ApiWorkspaceSchema;
export const ApiWorkspacesResponseSchema = z.array(ApiWorkspaceSchema);

export type ApiPoint = z.infer<typeof ApiPointSchema>;
export type ApiPointWithFacing = z.infer<typeof ApiPointWithFacingSchema>;
export type ApiMinionElement = z.infer<typeof ApiMinionElementSchema>;
export type ApiMinion = z.infer<typeof ApiMinionSchema>;
export type ApiMinionMessage = z.infer<typeof ApiMinionMessageSchema>;
export type ApiWorkspace = z.infer<typeof ApiWorkspaceSchema>;
