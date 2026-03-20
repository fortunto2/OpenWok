# Tailwind Migration — Acceptance Criteria

- All custom CSS classes in style.css replaced with Tailwind utility classes
- style.css reduced to ≤50 lines (only CSS variables + custom stuff Tailwind can't do)
- All 13 Dioxus components use Tailwind classes
- Mobile-first responsive (sm/md/lg breakpoints)
- Dark mode support via Tailwind dark: prefix
- `make tailwind && dx build --platform web` compiles
- Visual parity with current design (same colors, spacing, layout)
