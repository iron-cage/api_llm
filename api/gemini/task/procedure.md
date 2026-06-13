# Task Lifecycle Procedure

**Trigger**: A trackable work item is identified — bug, hygiene violation, feature gap, or improvement.

1. Check `task/unverified/readme.md` and `task/verified/readme.md` to find the highest existing NNN ID; use the next integer as the new task's NNN.
2. Create `task/unverified/NNN_name.md` following the tsk.rulebook.md task file format (Goal, In Scope, Out of Scope, Requirements, Work Procedure, Acceptance Criteria, Validation, History).
3. Register the new task in `task/unverified/readme.md` Responsibility Table.
4. Validation Gate: an independent reviewer (not the task author) checks all 4 Verification Dimensions and records a Verification Record in the task file.
5. On Verification Gate pass: move the task file from `task/unverified/` to `task/verified/`; update both subdirectory `readme.md` files.
6. Execute the task per its Work Procedure.
7. Record completion evidence in the task file's Verification Record section.
8. Move the task file from `task/verified/` to `task/completed/`; update both subdirectory `readme.md` files.
