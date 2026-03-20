# Tailwind CSS Migration

**Status:** [ ] Not Started
**Track:** tailwind-migration

## Context Handoff

**Intent:** Replace 471 lines of custom CSS with Tailwind utility classes. Keep visual design identical. Use `crates/frontend/assets/style.css` as reference for colors/spacing, then replace class-by-class.

**Key files:**
- `crates/frontend/src/main.rs` — 13 Dioxus components (1143 lines)
- `crates/frontend/assets/style.css` — current custom CSS (471 lines, CSS variables)
- `crates/frontend/input.css` — Tailwind entry point
- `crates/frontend/assets/tailwind.css` — generated output

**CSS variables to preserve as Tailwind theme:**
```
--primary: #e85d04  → orange-600ish
--primary-hover: #dc2f02 → red-600ish
--bg: #fafafa → gray-50
--surface: #ffffff → white
--text: #1a1a2e → gray-900
--text-muted: #6b7280 → gray-500
--border: #e5e7eb → gray-200
--success: #10b981 → emerald-500
--error: #ef4444 → red-500
--radius: 8px → rounded-lg
```

**Visual testing:** After each phase, start `dx serve` and use Playwright MCP to screenshot pages. Compare with current look.

---

- [ ] Task 1.1: Extend Tailwind theme in `input.css` — add custom colors (primary, primary-hover) as `@theme` block so Tailwind classes like `bg-primary` work. Keep CSS variables for fallback.
- [ ] Task 1.2: Migrate Layout component (header, nav, footer) — replace `.header`, `.nav`, `.nav-links`, `.logo`, `.content` classes with Tailwind. Remove corresponding CSS from style.css.
- [ ] Task 1.3: Migrate Home component — `.hero`, `.subtitle`, `.receipt-demo`, `.cta` → Tailwind classes. Remove from style.css.
- [ ] Task 1.4: Migrate RestaurantList + RestaurantCard — `.restaurants-grid`, `.restaurant-card`, `.card-header`, `.card-meta` → Tailwind grid/card layout.
- [ ] Task 1.5: Migrate RestaurantMenu + CartPanel — `.menu-grid`, `.menu-item`, `.cart`, `.cart-item`, `.cart-total` → Tailwind.
- [ ] Task 1.6: Migrate Checkout component — `.checkout`, `.receipt`, `.fee-line`, `.total`, `.address-input` → Tailwind form/receipt layout.
- [ ] Task 1.7: Migrate OrderTracking — `.tracking`, `.timeline`, `.status-step`, `.active-step` → Tailwind timeline.
- [ ] Task 1.8: Migrate PublicEconomicsPage — `.economics`, `.metrics-grid`, `.metric-card` → Tailwind.
- [ ] Task 1.9: Migrate OperatorConsole + MetricsPanel — `.operator`, `.orders-table`, `.metrics-panel` → Tailwind tables/panels.
- [ ] Task 1.10: Add dark mode — wrap key colors with `dark:` variants. Add toggle button in nav.
- [ ] Task 1.11: Clean up style.css — remove all migrated classes. Keep only what Tailwind can't express (animations, complex selectors). Target ≤50 lines.
- [ ] Task 1.12: Run `make tailwind && cargo check -p openwok-frontend`. Visual test all pages. `make check`. Commit.
