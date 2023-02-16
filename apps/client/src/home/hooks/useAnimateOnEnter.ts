import { useEffect } from "react";

export function useAnimateOnEnter() {
  useEffect(() => {
    // Set up a new observer
    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.target instanceof HTMLElement) {
          // Is the element in the viewport?
          if (entry.isIntersecting) {
            // Add the floatIn class:
            entry.target.classList.add("motion-safe:animate-floatInSlow");
          } else if (window.scrollY < entry.target.offsetTop) {
            // Otherwise remove the fadein class
            entry.target.classList.remove("motion-safe:animate-floatInSlow");
          }
        }
      });
    });

    // Get all the elements you want to show on scroll
    const targets = document.querySelectorAll(".show-on-scroll");

    // Loop through each of the target
    targets.forEach(function (target) {
      // Add the element to the watcher
      observer.observe(target);
    });

    return () => {
      observer.disconnect();
    };
  }, []);
}
