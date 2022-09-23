import { useState } from "react";
import { BehaviorSubject } from "rxjs";

import { useSubscribe } from "./useSubscribe";

export function useSubscribeValue<T>(
  subject: BehaviorSubject<T> | undefined | null
) {
  const [value, setValue] = useState<T | null>(subject?.value ?? null);

  useSubscribe(subject, setValue);

  return value;
}
