export function parseJWT(token: string): {
  id: string;
  role: string;
  exp: number;
  iat: number;
} {
  const base64Url = token.split(".")[1];
  if (!base64Url) throw new Error("Invalid token");
  const base64 = base64Url.replace("-", "+").replace("_", "/");
  return JSON.parse(Buffer.from(base64, "base64").toString("binary"));
}
