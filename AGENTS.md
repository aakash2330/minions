# Project Instructions

## Frontend Component Shape

- Keep server data in React Query. Use Zustand for UI state only, such as which panel is open and what it should show.
- The left navigation is the sidebar. The right-side sheet-style surface is the panel. Do not name right-panel code `sidebar`.
- Feature renderers should receive data and render only their domain UI. For example, `SessionChat` should accept a `Session` and render chat messages without fetching data, reading stores, or knowing whether it is inside a panel, card, or page.
- The app should mount one actual panel sheet through `AppPanel`, which owns `Panel.Root` and `Panel.Content`. `PanelRenderer` should render only the inner content based on panel state.
- Container renderers own orchestration. For example, `PanelRenderer` reads panel content state, fetches backend data through React Query as needed, chooses the content case, and passes data into feature renderers. Panel content components should not return `Panel.Root` or `Panel.Content`.
- Shared layout components should provide a default component for the common case and primitives for custom layouts. For panel content, prefer `<Panel title description>...</Panel>` inside the global panel content for normal cases. Use `Panel.Header`, `Panel.Title`, `Panel.Description`, and `Panel.Body` when the order or styling needs to be rearranged.

## Frontend API Shape

- Keep feature API splits minimal and role-based. Prefer a public API/orchestration file, a schemas file, and a mappers file before adding deeper folders.
- Public API files, such as `src/features/sessions/api/sessions.ts`, should own exported fetch functions, React Query keys, backend calls, validation orchestration, and public re-exports needed by current callers.
- Schema files, such as `sessionSchemas.ts`, should own Zod API response schemas and inferred API DTO types. Use Zod transforms for local DTO cleanup, such as backend field-name normalization.
- Mapper files, such as `sessionMappers.ts`, should own frontend/domain output types and API-to-domain assembly, especially when mapping joins multiple backend responses or depends on game/UI domain helpers.
