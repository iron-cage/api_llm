# Pitfall Entity Procedure

## Trigger

When a confirmed design pitfall is discovered in `api_huggingface` through task investigation, code review, or production incident — a pitfall is confirmed when root cause is known and avoidance pattern is established.

## Prerequisites

| Condition | Verification |
|-----------|--------------|
| Root cause identified and verified | Root cause explanation written, reviewed against codebase |
| Avoidance pattern established | Avoidance guidance is concrete and actionable (not "be careful") |
| Pitfall is distinct from all existing pitfall/ instances | `pitfall/readme.md` Overview Table reviewed — no overlap |

## Procedure Steps

1. Assign the next sequential ID from `pitfall/readme.md` Overview Table.
2. Create `pitfall/NNN_name.md` with H1 `# Pitfall: {Name}`, `### Scope` (4 bullets), then type-specific sections: `### Symptom`, `### Root Cause`, `### Avoidance Pattern`, `### Detection`.
3. Add typed reference sections for all doc entities that relate to this pitfall (sources, tests, and any API/feature instances where the pitfall manifests).
4. Add a row to `pitfall/readme.md` Overview Table: `| NNN | [Name](NNN_name.md) | One-line pitfall summary | ✅ |`.
5. Add a row to `docs/entities/readme.md` Instances Table for the new pitfall instance.
6. Create `tests/docs/pitfall/NN_name.md` with GWT scenarios (PF-prefix) verifying the avoidance pattern is in effect.
7. Add the spec file to the new instance's `### Tests` typed reference section.
8. Add a cross-reference from any related api/ or feature/ instance to this pitfall if the relationship is material.

## Expected Outcome

A new pitfall/ instance exists documenting the confirmed pitfall, the master file and entity index are updated, and a paired GWT spec file verifies the avoidance pattern is enforced in source.

## Rollback Procedure

Delete the instance file. Remove its row from `pitfall/readme.md` Overview Table and `docs/entities/readme.md` Instances Table. Delete the paired spec file in `tests/docs/pitfall/`. Remove any cross-references added to related api/ or feature/ instances.
