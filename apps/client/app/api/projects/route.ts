import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { CreateProjectResponse, schema } from "./types";

// Create a new project
export async function POST(request: NextRequest) {
  const session = await getUserSession();
  if (!session || !session.user.address) return new Response("Unauthorized", { status: 401 });

  const { title } = schema.parse(await request.json());

  const publicId = nanoidShort();
  await prisma.project.create({ data: { publicId, owner: session.user.address, title } });

  const json: CreateProjectResponse = { id: publicId };
  return NextResponse.json(json);
}
