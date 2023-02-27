import { CreateProjectArgs, CreateProjectResponse } from "./types";

export async function createProject(name?: string) {
  const args: CreateProjectArgs = { name };
  const response = await fetch("/api/project", {
    method: "POST",
    body: JSON.stringify(args),
    headers: { "Content-Type": "application/json" },
  });
  const { id } = (await response.json()) as CreateProjectResponse;
  return id;
}
