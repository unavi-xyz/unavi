import { PutParams } from "./types";

export async function linkProject(id: string, args: PutParams) {
  const res = await fetch(`/api/projects/${id}/link`, {
    method: "PUT",
    body: JSON.stringify(args),
  });
  if (!res.ok) throw new Error(res.statusText);
}
