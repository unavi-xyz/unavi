import { initTRPC, TRPCError } from "@trpc/server";

import { Context } from "./context";

const t = initTRPC.context<Context>().create({
  errorFormatter({ shape }) {
    return shape;
  },
});

export const mergeRouters = t.mergeRouters;
export const router = t.router;

const isAuthed = t.middleware(({ next, ctx }) => {
  if (!ctx.session || !ctx.session.address) throw new TRPCError({ code: "UNAUTHORIZED" });

  return next({
    ctx: {
      session: {
        ...ctx.session,
        address: ctx.session.address,
      },
    },
  });
});

export const publicProcedure = t.procedure;
export const protectedProcedure = t.procedure.use(isAuthed);
