import { Middleware, RequestContext } from "lucia-auth";
import { NextRequest, NextResponse } from "next/server";

/**
 * Lucia auth middleware for Next.js 13 Edge
 * {@see https://github.com/pilcrowOnPaper/lucia/discussions/548}
 */
export function edge(): Middleware<[NextRequest, NextResponse | undefined]> {
  return (incoming, outgoing, env) => {
    const requestContext = {
      request: {
        url: getUrl(incoming, env),
        method: incoming.method,
        headers: {
          origin: incoming.headers.get("origin"),
          cookie: incoming.headers.get("cookie"),
        },
      },
      setCookie: (cookie) => {
        outgoing?.cookies.set(cookie.name, cookie.value, cookie);
      },
    } as const satisfies RequestContext;

    return requestContext;
  };
}

function getUrl(incoming: NextRequest, env: "DEV" | "PROD") {
  if (!incoming.headers.get("host")) return "";

  const protocol = env === "DEV" ? "http:" : "https:";
  const host = incoming.headers.get("host");
  const pathname = incoming.url;

  return `${protocol}//${host}${pathname}`;
}
