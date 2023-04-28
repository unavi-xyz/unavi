import { Middleware, RequestContext } from "lucia-auth";
import { NextRequest, NextResponse } from "next/server";

/**
 * Lucia auth middleware for Next.js 13 Edge
 * {@see https://github.com/pilcrowOnPaper/lucia/discussions/548}
 */
export function edge(): Middleware<[NextRequest, NextResponse | undefined]> {
  return (incoming, outgoing) => {
    const requestContext = {
      request: {
        url: incoming.url,
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
