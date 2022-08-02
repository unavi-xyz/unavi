import { PrismaClient } from "@prisma/client";
import { randomBytes } from "crypto";

export const prisma = new PrismaClient();

async function createSecret() {
  const internal = await prisma.internal.findFirst();
  if (internal === null) {
    const secret = randomBytes(64).toString("hex");
    await prisma.internal.create({ data: { secret } });
  }
}

createSecret();
