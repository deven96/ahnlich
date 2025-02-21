import type { Config } from "tailwindcss";

export default {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        tertiary: '#df0816',
        primary: '#1589ba',
        secondary: '#09b5ca',
        grey: {
          1: '#e0e2e3',
          2: '#767675',
          3: '#494848',
          4: '#1c1c1c'
        },
        forest: {
          1: '#296e72',
          2: '#144242',
          3: '#204e63'
        }
      }
    },
  },
  plugins: [],
} satisfies Config;

  