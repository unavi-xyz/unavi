export const DISCORD_URL = "https://discord.gg/VCsAEneUMn";
export const TWITTER_URL = "https://twitter.com/TheWiredXR";
export const GITHUB_URL = "https://github.com/wired-labs/wired";

export const DOCS_URL =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3100"
    : "https://docs.thewired.space";

export enum SessionStorage {
  AutoLogin = "auto_login",
}
