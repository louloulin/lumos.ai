/** @type {import('tailwindcss').Config} */

module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
    "./index.html",
    "./src/components/**/*.{js,jsx,ts,tsx}",
    "./src/pages/**/*.{js,jsx,ts,tsx}",
    "./src/layouts/**/*.{js,jsx,ts,tsx}",
    "./src/domains/**/*.{js,jsx,ts,tsx}"
  ],
  theme: {
    extend: {
      animation: {
        'fade-in': 'fade-in 1s ease-out',
      },
      keyframes: {
        'fade-in': {
          '0%': {
            opacity: '0.8',
            backgroundColor: 'hsl(var(--muted))',
          },
          '100%': {
            opacity: '1',
            backgroundColor: 'transparent',
          },
        },
      },
    },
  },
  plugins: [
    require('tailwindcss-animate'),
  ],
};
