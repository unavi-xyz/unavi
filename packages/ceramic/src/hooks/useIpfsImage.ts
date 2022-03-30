import { useEffect, useState } from "react";
import { useIpfsFile } from "./useIpfsFile";

export function useIpfsImage(cid: string) {
  const [image, setImage] = useState<string>();

  const { file } = useIpfsFile(cid);

  useEffect(() => {
    if (file) fileToDataUrl(file).then(setImage);
    else setImage(undefined);
  }, [file]);

  return image;
}

function fileToDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();

    reader.addEventListener("loadend", (e) => {
      if (e.target) resolve(e.target.result as string);
    });
    reader.addEventListener("error", reject);

    reader.readAsDataURL(file);
  });
}
