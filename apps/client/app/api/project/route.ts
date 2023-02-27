import { customAlphabet } from "nanoid";
import { NextResponse } from "next/server";

import { getServerSession } from "../../../src/server/helpers/getServerSession";
import { prisma } from "../../../src/server/prisma";
import { PROJECT_ID_LENGTH } from "./constants";
import { CreateProjectResponse, schema } from "./types";

const nanoid = customAlphabet(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
  PROJECT_ID_LENGTH
);

// Create a new project
export async function POST(request: Request) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Not authenticated");

  const { name } = schema.parse(await request.json());
  const id = nanoid();

  await prisma.project.create({
    data: { id, owner: session.address, name: name ?? "", description: "" },
  });

  const json: CreateProjectResponse = { id };
  return NextResponse.json(json);
}
