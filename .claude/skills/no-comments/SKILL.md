---
name: no-comments
description: Comment convention for this repository: code carries no comments, except in the specific cases listed here. Use ALWAYS when writing, editing, refactoring or reviewing any source file (.rs, .sql, .toml, .ts, .py, etc.) in this project, and when deciding whether a comment should be added, kept or removed.
---

# Code without comments

The default in this repository is **no comments**. That includes doc comments
(`///`, `//!`, docstrings, JSDoc). An item being public is not a reason to document it.

Code should explain itself through the names of things. A comment is debt: it ages, it lies
after a refactor, and nobody tests it.

## The test before writing a comment

> If I delete this comment, will someone break the code or lose time?

**No** → delete it. **Yes** → write it, and explain the *why*, never the *what*.

## Exceptions: when the comment stays

Only these. If the case is not on the list, it is not an exception.

1. **A non-obvious constraint the code does not express.** A version pin, an order of
   operations that looks arbitrary, a workaround for a third-party bug. Without the
   comment, someone "fixes" it and breaks it.

   ```toml
   # 2.x drops the MySQL 5.7 wire protocol, which the staging database still runs.
   some-driver = ">=1.4, <2"
   ```

2. **Directives a tool requires**, which only work as comments:
   `// SAFETY:` on an `unsafe` block, `// eslint-disable-next-line`, `# type: ignore`,
   `# noqa`, `// TODO(#123)`. Attributes such as `#[allow(...)]` are not comments and are
   used normally.

3. **An external reference** that defines a format and does not fit in a name: a link to an
   EIP, RFC, issue or spec section.

4. **A legal / license header** required by the project or a dependency.

5. **An explicit request from the user** for that task.

## What never justifies a comment

- Restating the code: `// increment the counter`, `// return the result`.
- A doc comment on a public item just because it is public.
- A section separator: `// ---- helpers ----`.
- Commented-out code. Delete it; the history is in git.
- Making up for a bad name. Rename it.
- Narrating the steps of a function (`// 1. fetch`, `// 2. convert`, `// 3. save`).
  If it needs a script, extract functions with those names.

## Instead of commenting

| Urge to comment | Do this |
|---|---|
| Explain what a block does | Extract a function with that name |
| Explain a magic value | Named constant |
| Explain a parameter | Rename it, or use a type that only accepts valid values |
| Explain an edge case | Write a test named after the case |
| Explain why the change was made | Commit message / PR description |

## When editing existing code

- Do not add new comments that do not fall under the exceptions.
- Do not remove existing comments outside the scope of what was asked.
  Comment cleanup is a task of its own, not a side effect.
- If you change the code a comment describes, update or delete the comment along with it.
  An outdated comment is worse than no comment.
