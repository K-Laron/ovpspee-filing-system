import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: '#C0392B',
        secondary: '#2C3E50',
        background: '#F5F6FA',
        surface: '#FFFFFF',
        border: '#E2E8F0',
        muted: '#64748B',
        success: '#38A169',
        warning: '#DD6B20'
      },
      fontFamily: {
        sans: ['Inter', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'Consolas', 'monospace']
      }
    }
  },
  plugins: []
} satisfies Config;
