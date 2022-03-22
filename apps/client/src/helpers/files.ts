export async function dataUrlToFile(dataUrl: string, name: string) {
  const res = await fetch(dataUrl);
  const blob = await res.blob();
  return new File([blob], name);
}

export function fileToDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();

    reader.addEventListener("loadend", (e) =>
      resolve(e.target.result as string)
    );
    reader.addEventListener("error", reject);

    reader.readAsDataURL(file);
  });
}
