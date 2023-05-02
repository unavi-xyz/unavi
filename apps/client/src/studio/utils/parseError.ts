export const ERROR_MESSAGE = {
  UNAUTHORIZED: "Unauthorized.",
  USER_REJECTED_TRANSACTION: "User rejected transaction.",
} as const;

/**
 * Parses an error and return a string to display to the user.
 * @param err The error to process.
 * @param fallbackMessage The message to display if the error cannot be processed.
 */
export function parseError(err: unknown, fallbackMessage = ""): string {
  if (err instanceof Error) return parseErrorMessage(err.message);

  if (hasMessage(err)) return parseErrorMessage(err.message);

  if (typeof err === "string") return parseErrorMessage(err);

  return fallbackMessage;
}

function parseErrorMessage(message: string) {
  const lowercase = message.toLowerCase();

  if (lowercase.includes("user rejected transaction"))
    return ERROR_MESSAGE.USER_REJECTED_TRANSACTION;
  if (lowercase.includes("unauthorized")) return ERROR_MESSAGE.UNAUTHORIZED;

  return message;
}

function hasMessage(err: unknown): err is { message: string } {
  return typeof err === "object" && err !== null && "message" in err;
}
