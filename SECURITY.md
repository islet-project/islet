# Security Policy

## Supported Versions

The Islet project actively maintains and evaluates security reports on our primary active codebase.

We accept and investigate security reports targeting:
* **The latest stable release** (the most recently tagged release on the `main` branch).
* **The active development environment** (any commits on the `main` branch newer than the latest tagged release).

Security reports targeting older, deprecated, or unmaintained versions will not be prioritized unless they directly affect the current active development branch. We recommend verifying issues against the latest commit on `main` prior to submission.

---

## Reporting a Vulnerability

The Islet project is committed to maintaining a secure environment. If you identify a potential security vulnerability—or an issue you suspect may have security implications—we request that you report it through one of our private channels below rather than opening a public issue. This helps us collaborate on a fix and protect users before the details are made public.

If you are uncertain whether an issue has security implications, we still encourage utilizing these private channels. Our team will review the report and help determine the appropriate classification.

### Option 1: GitHub Private Vulnerability Reporting (Preferred)
You can privately and securely report vulnerabilities directly through GitHub. This uses HTTPS/TLS to encrypt all communications:
👉 **[Report a vulnerability via GitHub Security Advisories](https://github.com/islet-project/islet/security/advisories/new)**

### Option 2: Email to Islet Security Team
Alternatively, you can email our dedicated project security address at **[islet@samsung.com](mailto:islet@samsung.com)**. When reporting, please include:
* A detailed description of the issue.
* Detailed steps to reproduce the vulnerability (including proof-of-concept code if available).
* Affected versions, specific commit hashes, and platforms.
* Any known workarounds or mitigations.

---

## Disclosure and Response Process

As an incubation-stage project maintained by a lean team, we take every security report seriously and will review submissions as quickly as our availability allows.

Once a vulnerability is reported:
1. **Acknowledgement:** The Islet development team will review the report and acknowledge receipt after initial validation.
2. **Triage:** We will investigate the issue and assess its impact on the supported codebase.
3. **Fix & Release:** We will develop a fix and coordinate a security release. Critical vulnerabilities will be prioritized.
4. **Recognition:** With the reporter's consent, we will credit their contribution in our release notes once the patch is publicly merged.
