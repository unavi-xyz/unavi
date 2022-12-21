export async function cropImage(url: string, ratio = 5 / 3): Promise<File> {
  const res = await fetch(url);
  const imageBlob = await res.blob();

  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Failed to get canvas context");

  const img = new Image();
  img.src = URL.createObjectURL(imageBlob);

  return await new Promise((resolve, reject) => {
    img.onload = () => {
      const width = img.width;
      const height = img.height;

      let cropWidth: number;
      let cropHeight: number;

      if (height > width) {
        cropWidth = width;
        cropHeight = cropWidth / ratio;
      } else {
        cropHeight = height;
        cropWidth = cropHeight * ratio;
      }

      // Zoom in if image is too small
      if (cropWidth > width) {
        cropWidth = width;
        cropHeight = cropWidth / ratio;
      } else if (cropHeight > height) {
        cropHeight = height;
        cropWidth = cropHeight * ratio;
      }

      const cropX = (width - cropWidth) / 2;
      const cropY = (height - cropHeight) / 2;
      canvas.width = cropWidth;
      canvas.height = cropHeight;

      ctx.drawImage(img, cropX, cropY, cropWidth, cropHeight, 0, 0, cropWidth, cropHeight);

      canvas.toBlob((blob) => {
        URL.revokeObjectURL(img.src);

        if (blob) resolve(new File([blob], "image.jpg", { type: "image/jpeg" }));
        else reject();
      }, "image/jpeg");
    };
  });
}
