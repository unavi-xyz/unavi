/**
 * @type {import('tailwindcss').Config}}}
 */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}", "./app/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      backgroundImage: {
        arrow: "url('/images/svg/arrow.svg')",
      },
      boxShadow: {
        dark: "0 1px 3px 0 rgb(0 0 0 / 0.3)",
      },
      dropShadow: {
        dark: "0 2px 16px rgba(0, 0, 0, 0.8)",
      },
      animation: {
        fadeIn: "fadeIn 150ms ease-in forwards",
        fadeInDelayed: "fadeIn 500ms ease-in forwards",
        fadeOut: "fadeOut 150ms ease-in forwards",
        floatIn: "floatIn 150ms ease-in forwards",
        floatInSlow: "floatIn 1s ease-in forwards",
        scaleIn: "scaleIn 150ms ease-in-out forwards",
        scaleOut: "scaleOut 150ms ease-in-out forwards",
        scaleInFull: "scaleInFull 200ms ease forwards",
        scaleOutFull: "scaleOutFull 200ms ease forwards",
        textScroll: "textScroll 5s ease infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: 0 },
          "100%": { opacity: 1 },
        },
        fadeInDelayed: {
          "0%": { opacity: 0 },
          "50%": { opacity: 0 },
          "100%": { opacity: 1 },
        },
        fadeOut: {
          "0%": { opacity: 1 },
          "100%": { opacity: 0 },
        },
        floatIn: {
          "0%": { opacity: 0, transform: "translateY(10px)" },
          "100%": { opacity: 1, transform: "translateY(0)" },
        },
        scaleIn: {
          "0%": { opacity: 0, transform: "scale(0.75)" },
          "100%": { opacity: 1, transform: "scale(1)" },
        },
        scaleOut: {
          "0%": { opacity: 1, transform: "scale(1)" },
          "100%": { opacity: 0, transform: "scale(0.75)" },
        },
        scaleInFull: {
          "0%": { transform: "scale(0)" },
          "100%": { transform: "scale(1)" },
        },
        scaleOutFull: {
          "0%": { transform: "scale(1)" },
          "100%": { transform: "scale(0)" },
        },
        textScroll: {
          "0%, 100%": {
            "background-size": "200% 200%",
            "background-position": "left center",
          },
          "50%": {
            "background-size": "200% 200%",
            "background-position": "right center",
          },
        },
      },
    },
  },
};
