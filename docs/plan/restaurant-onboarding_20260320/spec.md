# Specification: Restaurant Onboarding & Self-Service

**Track ID:** restaurant-onboarding_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Restaurant owners need to register their restaurants, manage menus, and control availability — all through self-service. Currently, restaurants exist only as seeded data with no ownership model. The `UserRole::RestaurantOwner` enum variant exists but is unused, and the `restaurants` table has no `owner_id` column.

This track adds: database ownership linkage, role-based auth guards, CRUD endpoints for restaurant management, and a frontend dashboard for restaurant owners. It builds on the existing Supabase Auth + JWT verification pipeline from the auth-payments track.

## Acceptance Criteria

- [ ] `restaurants` table has `owner_id` FK to `users(id)` + `description`, `address`, `phone`, timestamps
- [ ] Logged-in user can create a restaurant (auto-assigned RestaurantOwner role)
- [ ] Restaurant owner can edit name, description, address, phone, active status
- [ ] Restaurant owner can add, edit, and delete menu items
- [ ] `GET /api/my/restaurants` returns only restaurants owned by current user
- [ ] Non-owner cannot modify another owner's restaurant (403)
- [ ] Frontend has `/my-restaurants` dashboard, `/my-restaurants/:id` settings, menu editor
- [ ] All new endpoints work in both axum API and Cloudflare Worker
- [ ] Tests cover ownership enforcement and role-based access
- [ ] Deployed to CF Workers and verified

## Dependencies

- auth-payments_20260320 (Supabase Auth, JWT verification, User table) — DONE
- repo-abstraction_20260320 (Repository trait, handlers crate, D1Repo) — DONE

## Out of Scope

- KYB (Know Your Business) verification for restaurants
- Node operator approval workflow for new restaurants
- Restaurant analytics/reporting dashboard
- Image upload for restaurant logo or menu item photos
- Operating hours / scheduling
- Courier PWA (separate track)

## Technical Notes

- `UserRole::RestaurantOwner` already exists in `crates/core/src/types.rs` — just unused
- Auth flow exists: Supabase JWT → `AuthUser` extractor → `supabase_user_id` → user lookup
- Repository pattern: add methods to trait in `core/repo.rs`, implement in `api/sqlite_repo.rs` and `worker/d1_repo.rs`
- Handlers go in `crates/handlers/` (shared between axum API and worker)
- Worker can't impl Repository trait (D1Database is !Send) but uses same method signatures
- Frontend: Dioxus 0.6 SPA with custom CSS (no Tailwind yet), uses reqwest for API calls
- Role promotion: on restaurant creation, user role changes from Customer → RestaurantOwner
