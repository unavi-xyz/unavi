import { useEffect } from "react";
import { BehaviorSubject } from "rxjs";

export function useSubscribe<T>(
  subject: BehaviorSubject<T> | undefined | null,
  next: (value: T) => void
) {
  useEffect(() => {
    if (!subject) return;

    const subscription = subject.subscribe({ next });

    return () => subscription.unsubscribe();
  }, [subject, next]);
}
