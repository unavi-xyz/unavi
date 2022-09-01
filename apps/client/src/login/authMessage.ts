export function createAuthMessage(host: string, nonce: string) {
  const message = [
    "🔌 The Wired Login",
    `💻 Host: ${host}`,
    `⚡ Nonce: ${nonce}`,
  ].join("\n");
  return message;
}

export function parseAuthMessage(message: string) {
  const lines = message.split("\n");

  const hostLine = lines[1].split(":").slice(1).join(":");
  const nonceLine = lines[2].split(":").slice(1).join(":");

  const host = hostLine.trim();
  const nonce = nonceLine.trim();

  return { host, nonce };
}
