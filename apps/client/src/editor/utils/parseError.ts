/**
 * Parses an error and return a string to display to the user.
 * @param err The error to process.
 * @param fallbackMessage The message to display if the error cannot be processed.
 */
export function parseError(err: unknown, fallbackMessage: string): string {
  if (err instanceof Error) {
    if (err.message.includes("user rejected transaction")) return "User rejected transaction.";
    return err.message;
  }

  if (typeof err === "string") return err;

  return fallbackMessage;
}
