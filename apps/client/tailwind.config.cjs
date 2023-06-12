/**
 * @type {import('tailwindcss').Config}}}
 */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}", "./app/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      animation: {
        backgroundScroll: "backgroundScroll 5s ease infinite",
        fadeIn: "fadeIn 150ms ease-in forwards",
        fadeInDelayed: "fadeIn 500ms ease-in forwards",
        fadeOut: "fadeOut 150ms ease-in forwards",
        floatIn: "floatIn 150ms ease-in forwards",
        floatInSlow: "floatIn 1s ease-in forwards",
        scaleIn: "scaleIn 150ms ease-in-out forwards",
        scaleInFull: "scaleInFull 200ms ease forwards",
        scaleOut: "scaleOut 150ms ease-in-out forwards",
        scaleOutFull: "scaleOutFull 200ms ease forwards",
      },
      aspectRatio: {
        card: "5/3",
      },
      backgroundImage: {
        arrow: "url('/images/svg/arrow.svg')",
      },
      boxShadow: {
        dark: "0 2px 12px 0 rgb(0 0 0 / 0.2)",
      },
      dropShadow: {
        dark: "0 2px 16px rgba(0, 0, 0, 0.8)",
      },
      keyframes: {
        backgroundScroll: {
          "0%, 100%": {
            "background-position": "left center",
            "background-size": "200% 200%",
          },
          "50%": {
            "background-position": "right center",
            "background-size": "200% 200%",
          },
        },
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
        scaleInFull: {
          "0%": { transform: "scale(0)" },
          "100%": { transform: "scale(1)" },
        },
        scaleOut: {
          "0%": { opacity: 1, transform: "scale(1)" },
          "100%": { opacity: 0, transform: "scale(0.75)" },
        },
        scaleOutFull: {
          "0%": { transform: "scale(1)" },
          "100%": { transform: "scale(0)" },
        },
      },
    },
  },
};
