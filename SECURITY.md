# Security

## What Ordo can access

Ordo asks Google for two scopes: `gmail.modify` (read message headers, add or remove labels, move to trash) and `gmail.settings.basic` (create and delete filters). It never sends mail and never deletes messages permanently.

## Where the secrets live

- **Refresh token**: in the operating system keychain (Windows Credential Manager), never in a file.
- **Access token**: in memory only, renewed when it expires.
- **`credentials.json`**: in the app data folder (`%APPDATA%\com.lojan.ordo`). Google treats desktop client secrets as non confidential; the refresh token is what grants access.
- Nothing is sent anywhere except Google and your local Claude Code.

## Sign in

OAuth runs in your browser with PKCE (S256), a random `state` checked on return, a loopback listener on `127.0.0.1` with a random port and a 5 minute timeout.

## Claude

- Claude Code runs locally with no tools, no MCP servers, no user settings and no saved session, so it cannot read files or act.
- It receives only sender names, addresses, message counts and a few subjects, never message bodies.
- Its answer must match a JSON schema and is validated: only senders it was shown (or their domains, never public ones like gmail.com), sanitized label names, one label per sender.
- A malicious subject can at most change a proposal, which you review before anything is applied.

## Destructive actions

- Everything goes to the Gmail trash (recoverable for 30 days).
- Protected labels are excluded from every clean up query.
- Emptying, deleting and moving to trash ask for a second click.
- Input from the UI is checked again in the engine (label names, senders, ids) and invalid input stops the whole operation before Gmail is touched.

## Desktop hardening

- Strict Content Security Policy.
- Minimal Tauri capabilities; links can only open `https://*.google.com/*` and `https://github.com/*`.

## Known limits

- Builds are not code signed yet, so Windows SmartScreen may warn on first launch.
- With the Google OAuth app in testing mode the permission expires every 7 days.

## Reporting a vulnerability

Please open a private security advisory on the GitHub repository instead of a public issue.