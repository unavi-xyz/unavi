import { useState } from "react";
import { BehaviorSubject } from "rxjs";

import { useSubscribe } from "./useSubscribe";

export function useSubscribeValue<T>(
  subject: BehaviorSubject<T> | undefined | null,
  defaultValue: T | null = null
) {
  const [value, setValue] = useState<T | null>(defaultValue);

  useSubscribe(subject, setValue);

  return value;
}
