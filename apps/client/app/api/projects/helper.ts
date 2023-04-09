import { CreateProjectArgs, CreateProjectResponse } from "./types";

export async function createProject(title?: string) {
  const args: CreateProjectArgs = { title };
  const response = await fetch("/api/projects", {
    method: "POST",
    body: JSON.stringify(args),
    headers: { "Content-Type": "application/json" },
  });
  const { id } = (await response.json()) as CreateProjectResponse;
  return id;
}
