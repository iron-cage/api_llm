# Collection Entity Procedure

## Trigger

When a new enumerated item catalog is needed in `api_huggingface` documentation — e.g., a new feature flag set, a new tier classification table, or a new reference item list.

## Prerequisites

| Condition | Verification |
|-----------|--------------|
| A distinct catalog of enumerable items exists | At least 3 distinct items that share a classification scheme |
| No existing collection/ instance covers this catalog | `collection/readme.md` Overview Table reviewed — no overlap |
| The catalog is not a procedure (use operation/) or a contract (use feature/) | Content is purely enumerative, not procedural or contractual |

## Procedure Steps

1. Assign the next sequential ID from `collection/readme.md` Overview Table.
2. Create `collection/NNN_name.md` with H1 `# Collection: {Name}`, `### Scope` (4 bullets), `### Items` (table), `### Classification` (prose), then typed reference sections alphabetically with `### Sources` and `### Tests` last.
3. Populate `### Items` with a table covering all known items; group by tier or category if applicable.
4. Write `### Classification` prose describing how items are grouped and what membership in each group means.
5. Add typed reference sections for all doc entities that cross-reference this instance.
6. Add a row to `collection/readme.md` Overview Table: `| NNN | [Name](NNN_name.md) | Purpose | ✅ |`.
7. Add a row to `docs/entities/readme.md` Instances Table for the new instance.
8. Update `docs/readme.md` if the new instance warrants a mention in the Collections navigation section.
9. Create `tests/docs/collection/NN_name.md` with GWT scenarios (CL-prefix) covering item presence, classification correctness, and catalog completeness.
10. Add the spec file to the new instance's `### Tests` typed reference section.

## Expected Outcome

A new collection/ instance file exists, the master file and entity index are updated, and a paired GWT spec file in `tests/docs/collection/` covers the catalog contract.

## Rollback Procedure

Delete the instance file. Remove its row from `collection/readme.md` Overview Table, `docs/entities/readme.md` Instances Table, and `docs/readme.md` navigation. Delete the paired spec file in `tests/docs/collection/`. Remove any typed reference sections added to other doc instances for this collection.
