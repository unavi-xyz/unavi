import { useEffect, useState } from "react";

export function useMatrixContent(url: string) {
  const [content, setContent] = useState<null | string>(null);

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
