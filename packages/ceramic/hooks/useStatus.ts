import useSWR from "swr";
import { loader, Status } from "..";

export function useStatus(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const status = doc.content as Status;
    const controller = doc.metadata.controllers[0];
    return { status, controller };
  }

  const { data } = useSWR(`status-${id}`, fetcher);

  return data ?? { status: undefined, controller: undefined };
}
