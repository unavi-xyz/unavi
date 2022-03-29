export function fileToArrayBuffer(file: File) {
  return new Promise<ArrayBuffer>((resolve, reject) => {
    const reader = new FileReader();

    reader.addEventListener("loadend", (e) => {
      if (e.target) resolve(e.target.result as ArrayBuffer);
    });
    reader.addEventListener("error", reject);

    reader.readAsArrayBuffer(file);
  });
}

export function fileToDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();

    reader.addEventListener("loadend", (e) => {
      if (e.target) resolve(e.target.result as string);
    });
    reader.addEventListener("error", reject);

    reader.readAsDataURL(file);
  });
}
