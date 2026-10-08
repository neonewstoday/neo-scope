# Security Policy

## Supported versions

neo-scope is in **alpha**. Security fixes are made on `main` and shipped in the next
release; older releases are not patched.

| Version         | Supported |
| --------------- | --------- |
| latest release  | yes       |
| `main`          | yes       |
| older releases  | no        |

## Reporting a vulnerability

**Do not open a public issue, discussion or pull request for a security problem.**

Report it privately through one of these channels:

1. **GitHub private vulnerability reporting (preferred):** go to the
   [Security tab](https://github.com/neonewstoday/neo-scope/security) and click
   **Report a vulnerability**, or open
   <https://github.com/neonewstoday/neo-scope/security/advisories/new>.
2. **Email:** [lopes@neonewstoday.com](mailto:lopes@neonewstoday.com),
   [verv@neonewstoday.com](mailto:verv@neonewstoday.com) and
   [dean@neonewstoday.com](mailto:dean@neonewstoday.com). Write to all three, with
   `[neo-scope security]` in the subject.

Please include:

- The affected component (`neo-scope-indexer`, `neo-scope-api` or `neo-scope-db`) and the
  version or commit.
- A description of the issue and its impact.
- Steps to reproduce, or a proof of concept.
- Any suggested fix, if you have one.

## What to expect

- We acknowledge your report within **5 business days**.
- We investigate, keep you informed of progress, and agree on a disclosure date with you.
- Once a fix is released we publish a GitHub Security Advisory and credit you, unless you
  prefer to stay anonymous.

Please give us a reasonable time to fix the issue before disclosing it publicly.

## Scope

In scope is the code in this repository, for example:

- SQL injection or any request that escapes the API's read-only access.
- Requests that make the API exhaust memory, connections or CPU disproportionately.
- Malicious RPC responses that crash the indexer or corrupt indexed data.
- Secrets or credentials leaking through logs or API responses.

Out of scope:

- Deployments of neo-scope operated by third parties, and their configuration.
- Vulnerabilities in the Neo X network, its nodes or RPC providers. Report those to the
  respective projects.
- Vulnerabilities in dependencies that do not affect neo-scope. Report them upstream.
