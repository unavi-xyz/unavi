import { customAlphabet } from "nanoid";
import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { PROJECT_ID_LENGTH } from "./constants";
import { CreateProjectResponse, schema } from "./types";

const nanoid = customAlphabet(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
  PROJECT_ID_LENGTH
);

// Create a new project
export async function POST(request: NextRequest) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { title } = schema.parse(await request.json());

  const id = nanoid(PROJECT_ID_LENGTH);

  await prisma.project.create({
    data: { id, owner: session.address, title: title ?? "", name: "", description: "" },
  });

  const json: CreateProjectResponse = { id };
  return NextResponse.json(json);
}
