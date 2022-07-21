import { useEffect, useState } from "react";

interface Props {
  handle: FileSystemFileHandle;
}

export default function FileImageIcon({ handle }: Props) {
  const [image, setImage] = useState<string>();

  useEffect(() => {
    async function loadImage() {
      try {
        //get file
        const file = await handle.getFile();
        setImage(URL.createObjectURL(file));
      } catch (e) {
        console.error(e);
        setImage(undefined);
      }
    }

    loadImage();
  }, [handle]);

  if (!image) return null;

  return <img src={image} alt={handle.name} draggable={false} className="max-h-14 rounded-md" />;
}
