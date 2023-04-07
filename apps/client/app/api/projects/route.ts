import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { nanoid } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { CreateProjectResponse, schema } from "./types";

// Create a new project
export async function POST(request: NextRequest) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { title } = schema.parse(await request.json());

  const publicId = nanoid();
  await prisma.project.create({ data: { publicId, owner: session.address, title } });

  const json: CreateProjectResponse = { id: publicId };
  return NextResponse.json(json);
}
