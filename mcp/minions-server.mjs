#!/usr/bin/env node

const BACKEND_URL = process.env.MINIONS_BACKEND_URL ?? "http://127.0.0.1:8080";
const DEFAULT_SESSION_ID = process.env.MINIONS_SESSION_ID ?? "";
const TOOL_NAME = "perform_session_interaction";
const INTERACTION_TYPES = [
  "move-to-personal-table",
  "move-to-meeting-table",
  "turn-on-computer",
];

const tool = {
  name: TOOL_NAME,
  description:
    "Ask the current minion/session to perform a supported frontend interaction.",
  inputSchema: {
    type: "object",
    properties: {
      session_id: {
        type: "string",
        description:
          "Backend session id. Optional when controlling the current Codex session.",
      },
      interaction_type: {
        type: "string",
        enum: INTERACTION_TYPES,
        description:
          "Interaction type to perform, such as move-to-personal-table, move-to-meeting-table, or turn-on-computer.",
      },
    },
    required: ["interaction_type"],
    additionalProperties: false,
  },
};

let buffer = "";

process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;

  let newlineIndex;
  while ((newlineIndex = buffer.indexOf("\n")) !== -1) {
    const line = buffer.slice(0, newlineIndex).trim();
    buffer = buffer.slice(newlineIndex + 1);

    if (line.length === 0) {
      continue;
    }

    handleLine(line).catch((error) => {
      process.stderr.write(`minions MCP error: ${error.stack ?? error}\n`);
    });
  }
});

async function handleLine(line) {
  let message;

  try {
    message = JSON.parse(line);
  } catch {
    return;
  }

  if (Array.isArray(message)) {
    await Promise.all(message.map(handleMessage));
    return;
  }

  await handleMessage(message);
}

async function handleMessage(message) {
  switch (message.method) {
    case "initialize":
      sendResult(message.id, {
        protocolVersion: message.params?.protocolVersion ?? "2025-03-26",
        capabilities: {
          tools: {},
        },
        serverInfo: {
          name: "minions",
          version: "0.1.0",
        },
      });
      return;

    case "notifications/initialized":
    case "initialized":
      return;

    case "ping":
      sendResult(message.id, {});
      return;

    case "tools/list":
      sendResult(message.id, {
        tools: [tool],
      });
      return;

    case "tools/call":
      if (message.params?.name !== TOOL_NAME) {
        sendError(message.id, -32602, `Unknown tool: ${message.params?.name}`);
        return;
      }

      sendResult(message.id, await performSessionInteraction(message.params.arguments ?? {}));
      return;

    default:
      if (message.id !== undefined) {
        sendError(message.id, -32601, `Method not found: ${message.method}`);
      }
  }
}

async function performSessionInteraction(args) {
  const sessionId = normalizeText(
    args.session_id ?? args.sessionId ?? DEFAULT_SESSION_ID,
  );
  const interactionType = normalizeInteractionType(
    args.interaction_type ?? args.interactionType,
  );

  if (!sessionId) {
    return toolError("session_id is required");
  }

  if (!interactionType) {
    return toolError(
      `interaction_type must be one of: ${INTERACTION_TYPES.join(", ")}`,
    );
  }

  const response = await fetch(
    `${BACKEND_URL}/api/sessions/${encodeURIComponent(sessionId)}/interaction`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        interaction_type: interactionType,
      }),
    },
  );

  if (!response.ok) {
    return toolError(`backend returned ${response.status}`);
  }

  return {
    content: [
      {
        type: "text",
        text: `Performed ${interactionType} for ${sessionId}.`,
      },
    ],
  };
}

function normalizeText(value) {
  return typeof value === "string" ? value.trim() : "";
}

function normalizeInteractionType(value) {
  const normalized = normalizeText(value).toLowerCase().replaceAll("_", "-");

  if (INTERACTION_TYPES.includes(normalized)) {
    return normalized;
  }

  if (
    normalized === "personal" ||
    normalized === "personal-table" ||
    normalized === "personal table" ||
    normalized === "move to personal table"
  ) {
    return "move-to-personal-table";
  }

  if (
    normalized === "meeting" ||
    normalized === "meeting-table" ||
    normalized === "meeting table" ||
    normalized === "move to meeting table"
  ) {
    return "move-to-meeting-table";
  }

  if (
    normalized === "computer" ||
    normalized === "turn on computer" ||
    normalized === "turn-on computer"
  ) {
    return "turn-on-computer";
  }

  return null;
}

function toolError(text) {
  return {
    isError: true,
    content: [
      {
        type: "text",
        text,
      },
    ],
  };
}

function sendResult(id, result) {
  if (id === undefined) {
    return;
  }

  send({
    jsonrpc: "2.0",
    id,
    result,
  });
}

function sendError(id, code, message) {
  if (id === undefined) {
    return;
  }

  send({
    jsonrpc: "2.0",
    id,
    error: {
      code,
      message,
    },
  });
}

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}
