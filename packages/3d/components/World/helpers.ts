export async function dataUrlToFile(dataUrl: string) {
  const res = await fetch(dataUrl);
  const blob = await res.blob();
  return new File([blob], "preview");
}
