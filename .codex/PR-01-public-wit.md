0) Global rule for all repos (tell Codex this every time)

Use this paragraph at the top of every prompt:

Global policy: greentic:component@0.6.0 WIT must have a single source of truth in greentic-interfaces. No other repo should define or vendor package greentic:component@0.6.0 or world component-v0-v6-v0 in its own wit/ directory. Repos may keep tiny repo-specific worlds (e.g. messaging-provider-teams) but must depend on the canonical greentic component WIT via deps/ pointing at greentic-interfaces or via a published crate path, never by copying the WIT file contents.

E) Provider repos (greentic-messaging-providers, greentic-events-providers) prompt

These are the ones with many world.wit files. The trick is: they can keep their provider-specific worlds, but they should not re-declare the entire greentic component world; they should depend on it.

You are working in the greentic-messaging-providers repository (repeat similarly for greentic-events-providers).

Goal
- Provider-specific WIT worlds may remain (e.g. messaging-provider-teams world), but they must not re-copy or redefine the canonical `greentic:component@0.6.0` world shape.
- If provider worlds need to “include” the component world, do it via WIT deps referencing greentic-interfaces, not by copying WIT definitions.

Work
1) Inventory:
- Find all `.wit` files containing `package greentic:component@0.6.0;` AND defining `world component-v0-v6-v0 {`.
- Determine if these provider `world.wit` files are:
  a) defining their own provider world that *imports/uses* the canonical component world (preferred), or
  b) duplicating the canonical component world itself (must be removed).

2) Refactor provider WIT:
- Where a provider WIT file duplicates the canonical component world:
  - Remove the duplicated definitions.
  - Add a `deps/` folder (or WIT package dependency mechanism used in this repo) pointing at greentic-interfaces canonical path for `greentic:component@0.6.0`.
  - Update `use` statements to reference the canonical package.

3) Guest code:
- Update Rust guest code generation/build scripts to pull canonical WIT from greentic-interfaces.
- If any provider guest implementation needs to export v0.6 component world directly, use `greentic_interfaces_guest::export_component_v060!`.

4) Add a guard:
- Add a test or CI script that fails if any committed `.wit` file defines `world component-v0-v6-v0` under `package greentic:component@0.6.0` outside of greentic-interfaces.
  - Allowed: provider-specific worlds under other packages (e.g. `package greentic:messaging-provider-...`)
  - Not allowed: redefining the canonical greentic component world.

Deliverables
- Provider repos depend on canonical component WIT via deps; no copied canonical world definitions.
- Guard test/CI check added.

Now implement it.