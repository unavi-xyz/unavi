export function jsonArrayAdd(array: any, value: any) {
  const newArray = array ? [...Object.values(array), value] : [value];
  return newArray;
}

export function jsonArrayRemove(array: any, value: any) {
  const newArray = Object.values(array).filter((id) => id !== value);
  return newArray;
}
