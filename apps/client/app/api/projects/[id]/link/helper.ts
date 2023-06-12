import { PutParams } from "./types";

export async function linkProject(id: string, args: PutParams) {
  const res = await fetch(`/api/projects/${id}/link`, {
    body: JSON.stringify(args),
    method: "PUT",
  });
  if (!res.ok) throw new Error(res.statusText);
}
