import { useEffect, useState } from "react";

export function useMatrixContent(url: string | undefined) {
  const [content, setContent] = useState<string | null>(null);

  useEffect(() => {
    if (!url) {
      setContent(null);
      return;
    }

    const trimmed = url.replace("mxc://", "");
    const server = trimmed.split("/")[0];
    const download = `https://${server}/_matrix/media/r0/download/${trimmed}`;
    setContent(download);
  }, [url]);

  return content;
}
