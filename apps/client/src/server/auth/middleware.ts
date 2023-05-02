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
        headers: {
          cookie: incoming.headers.get("cookie"),
          origin: incoming.headers.get("origin"),
        },
        method: incoming.method,
        url: incoming.url,
      },
      setCookie: (cookie) => {
        outgoing?.cookies.set(cookie.name, cookie.value, cookie);
      },
    } as const satisfies RequestContext;

    return requestContext;
  };
}
