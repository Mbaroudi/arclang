// The environment the ArcLang MCP server is started with (mcp.ts). The server
// reads and writes under its workspace (ARCLANG_WORKSPACE) and under the
// directories named in ARCLANG_MCP_ROOTS, and refuses a path elsewhere: an
// agent can be asked for any path by what it read. The folders that are open
// are where it works; a variable the user set themselves is left as it is.

export interface McpSetting {
  /** The folders that are open, as file paths, in their order. */
  folders: string[];
  /** The compiler to use, when its path is absolute. */
  binary: string | null;
  /** The environment the server inherits: what is set there is the user's choice. */
  inherited: Record<string, string | undefined>;
  /** What separates the directories of ARCLANG_MCP_ROOTS: path.delimiter. */
  delimiter: string;
}

/** Any path, as before the server kept to its workspace. */
export const ANYWHERE = '*';

export function mcpEnvironment({ folders, binary, inherited, delimiter }: McpSetting): Record<string, string> {
  const env: Record<string, string> = {};
  if (binary) env.ARCLANG_BINARY = binary;
  if (!inherited.ARCLANG_WORKSPACE && folders.length > 0) env.ARCLANG_WORKSPACE = folders[0];
  if (inherited.ARCLANG_MCP_ROOTS) return env;
  // Without a folder there is no workspace: a file opened alone can be anywhere.
  if (folders.length === 0) return { ...env, ARCLANG_MCP_ROOTS: ANYWHERE };
  const named = folders.filter((folder) => !folder.includes(delimiter));
  return named.length > 0 ? { ...env, ARCLANG_MCP_ROOTS: named.join(delimiter) } : env;
}
