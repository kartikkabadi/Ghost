# Security Policy

## Supported Versions

Only the latest commit on `main` is supported for security updates.

## Reporting a Vulnerability

If you discover a security vulnerability in Ghost, please report it by emailing the repository owner through the contact information on the [GitHub profile](https://github.com/kartikkabadi).

Please include:
- A description of the vulnerability
- Steps to reproduce
- Affected files or components
- Possible impact

Do not open a public issue for security vulnerabilities.

## API Keys and Credentials

Ghost does not ship with API keys. All provider credentials must be supplied through environment variables listed in `.env.example`. If you accidentally commit a secret, rotate it immediately and remove it from your own `.env` file.

## Responsible Use

Ghost is designed for lawful, defensive open-source intelligence (OSINT) and research workflows. Use it only in compliance with [ETHICAL_USE.md](./ETHICAL_USE.md) and all applicable laws.
