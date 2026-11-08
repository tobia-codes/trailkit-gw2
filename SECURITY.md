# Security policy

## Reporting a vulnerability

Please **do not open a public issue** for a security problem. Report it
privately through GitHub:
**Security → Report a vulnerability** in this repository
(<https://github.com/tobia-codes/trailkit-gw2/security/advisories/new>).

Include what is affected (addon or tool and its version), how to reproduce it
and what an attacker could do. Do not include real API keys, notes or other
personal data; describe them instead.

You will get an answer as soon as possible. This is a hobby project run by one
person, so there is no guaranteed response time. Once a fix is released, the
advisory is published and you are credited unless you prefer otherwise.

## Supported versions

Only the latest release of each addon receives security fixes.

## Scope

In scope: the addons in `addons/`, the shared crates, the tools and the
service in this repository, and the data files they publish. Especially
relevant:

- anything that leaks a player's GW2 API key or note content;
- code that runs in the game process and can crash it or be abused through
  data the addon downloads;
- personal data that is collected, logged or sent against the
  [privacy rules](CONTRIBUTING.md#7-privacy-and-security).

Out of scope: Guild Wars 2, Raidcore Nexus and other addons; report those to
their authors.
