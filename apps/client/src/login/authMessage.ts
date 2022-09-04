export function createAuthMessage(
  address: string,
  host: string,
  uri: string,
  chain: number,
  nonce: string
) {
  const message = [
    "🔌 Sign in to the Wired",
    "",
    `👋 Address: ${address}`,
    `☁️ Host: ${host}`,
    `📜 URI: ${uri}`,
    `⛓️ Chain: ${chain}`,
    `⏰ Timestamp: ${new Date().toISOString()}`,
    `⚡ Nonce: ${nonce}`,
    `⚙️ Version: 1`,
  ].join("\n");
  return message;
}

export function parseAuthMessage(message: string) {
  function getLineValue(index: number) {
    const lines = message.split("\\n");
    const line = lines[index];
    // Get the value after the colon
    const value = line.split(":").slice(1).join(":").trim();
    return value;
  }

  const address = getLineValue(2);
  const host = getLineValue(3);
  const uri = getLineValue(4);
  const chain = parseInt(getLineValue(5));
  const timestamp = getLineValue(6);
  const nonce = getLineValue(7);
  const version = parseInt(getLineValue(8));

  return {
    address,
    host,
    uri,
    chain,
    timestamp,
    nonce,
    version,
  };
}
