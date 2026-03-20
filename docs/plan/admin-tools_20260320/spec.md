# Specification: Admin Tools (Block/Unblock + Dispute Resolution)

**Track ID:** admin-tools_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Add operational admin tools for the node operator: ability to block/unblock users (restaurants, couriers, customers) and a dispute resolution system tied to orders. This closes the remaining Phase 7 acceptance criterion: "Admin can block/unblock and resolve disputes."

The NodeOperator role already exists (`UserRole::NodeOperator`) but has no admin-specific endpoints or guards. The operator console (`/operator`) shows metrics and orders but cannot manage users or disputes. This track adds the missing backend endpoints, domain types, migration, and frontend tabs.

## Acceptance Criteria

- [x] Migration adds `blocked` column to `users` table and creates `disputes` table
- [~] Blocked users receive 403 on all authenticated endpoints — enforced on admin + dispute endpoints only, not on restaurant/courier/order endpoints
- [x] NodeOperator can list all users via `GET /api/admin/users`
- [x] NodeOperator can block/unblock a user via `PATCH /api/admin/users/{id}/block`
- [x] Customer can create a dispute on an order via `POST /api/orders/{id}/dispute`
- [x] NodeOperator can list disputes via `GET /api/admin/disputes`
- [x] NodeOperator can resolve a dispute via `PATCH /api/admin/disputes/{id}/resolve`
- [x] Operator Console has "Users" tab (list, block/unblock toggle)
- [x] Operator Console has "Disputes" tab (list, resolve action)
- [x] All admin endpoints require NodeOperator role (return 403 otherwise)
- [x] Tests cover block/unblock logic, dispute lifecycle, and auth guards

## Dependencies

- Existing auth system (Supabase JWT + `AuthUser` extractor)
- Existing `UserRole` enum and `update_user_role` repo method
- Existing operator console frontend (`/operator`)

## Out of Scope

- Refund flow on dispute resolution (future: integrate with Stripe refund API)
- Email/push notifications for disputes
- Dispute escalation or multi-level review
- Admin role separate from NodeOperator (use NodeOperator for MVP)

## Technical Notes

- **Auth guard pattern:** Look up the user by `supabase_user_id` from `AuthUser`, check `role == NodeOperator`. Return 403 if not. Also check `blocked == false`.
- **Blocked user enforcement:** Add a check in the `AuthUser` extractor flow or as middleware — after JWT validation, load user from DB, reject if blocked.
- **Dispute model:** `DisputeId`, `DisputeStatus` (Open, Resolved, Dismissed), linked to `order_id` + `user_id` (reporter), with `reason` text and `resolution` text.
- **Repository trait:** Add 5 new methods: `list_users`, `set_user_blocked`, `create_dispute`, `list_disputes`, `resolve_dispute`.
- **D1Repo parity:** Worker crate needs same methods (same signatures, manual impl since D1Database is !Send).
