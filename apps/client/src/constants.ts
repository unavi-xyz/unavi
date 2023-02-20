export const DISCORD_URL = "https://discord.gg/cazUfCCgHJ";
export const TWITTER_URL = "https://twitter.com/wired_xr";
export const GITHUB_URL = "https://github.com/wired-labs/wired";

const PROD_DOCS_URL = "https://docs.thewired.space";
const DEV_DOCS_URL = "http://localhost:3100";

export const DOCS_URL = process.env.NODE_ENV === "production" ? PROD_DOCS_URL : DEV_DOCS_URL;
