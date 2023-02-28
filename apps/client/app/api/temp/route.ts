import { nanoid } from "nanoid";
import { NextResponse } from "next/server";

import { getTempUpload } from "../../../src/server/s3/temp";

// Get temp upload URL
export async function GET() {
  const fileId = nanoid();
  const url = await getTempUpload(fileId);
  return NextResponse.json({ url, fileId });
}
