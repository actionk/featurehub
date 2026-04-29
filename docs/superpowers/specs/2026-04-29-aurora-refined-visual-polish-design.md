# Aurora Refined Visual Polish Design

## Goal

Refine Feature Hub's existing Aurora Glass visual direction without changing the product structure or core workflows.

## Chosen Direction

The approved direction is Aurora Refined: keep the current dark glass identity, but reduce visual noise, improve hierarchy, and make shared surfaces feel consistent across the shell.

## Scope

This pass focuses on the current shell and shared visual system:

- Adjust the global palette so purple remains a brand accent, while cyan and blue provide clearer secondary contrast.
- Reduce glow intensity and excessive decorative treatment on routine surfaces.
- Normalize glass panels, cards, buttons, tabs, board columns, feature rows, sessions rail, modals, and dropdowns.
- Tighten inconsistent spacing where panels currently feel mismatched.
- Improve selected, active, running, pending, and hover states so state communicates more strongly than decoration.
- Keep layouts and workflows intact.

## Non-Goals

- No full layout redesign.
- No new navigation model.
- No new data behavior.
- No broad component rewrites.
- No Tailwind utility migration.

## Implementation Notes

The codebase already centralizes most visual styling in `src/app.css`, and recent commits introduced Aurora primitives there. The safest implementation is a CSS-first pass that updates tokens and shared primitives, then adds small targeted overrides for shell areas that still use older card/button/tab rules.

Testing is visual and integration-oriented. Since this is styling, verification should prioritize `npm run build` and a manual browser/app inspection over brittle unit tests for exact colors.

