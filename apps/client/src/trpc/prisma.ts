import { PrismaClient } from "@prisma/client";
import { randomBytes } from "crypto";

declare global {
  // allow global `var` declarations
  // eslint-disable-next-line no-var
  var prisma: PrismaClient | undefined;
}

export const prisma = global.prisma || new PrismaClient();

if (process.env.NODE_ENV !== "production") global.prisma = prisma;

async function createSecret() {
  const internal = await prisma.internal.findFirst();
  if (internal === null) {
    const secret = randomBytes(64).toString("hex");
    await prisma.internal.create({ data: { secret } });
  }
}

createSecret();
