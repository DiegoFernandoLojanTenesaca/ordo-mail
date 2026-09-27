# Security

## What Ordo can access

Ordo asks Google for two scopes: `gmail.modify` (read message headers, add or remove labels, move to trash) and `gmail.settings.basic` (create and delete filters). It never sends mail and never deletes messages permanently.

## Where the secrets live

- **Refresh token and AI API keys**: in the operating system keychain (Windows Credential Manager), never in a file. Keys are write only from the interface: Ordo shows whether one is saved but never sends it back to the UI.
- **Access token**: in memory only, renewed when it expires.
- **`credentials.json`**: in the app data folder (`%APPDATA%\com.lojan.ordo`). Google treats desktop client secrets as non confidential; the refresh token is what grants access.
- Nothing is sent anywhere except Google and the AI engine you choose.

## Sign in

OAuth runs in your browser with PKCE (S256), a random `state` checked on return, a loopback listener on `127.0.0.1` with a random port and a 5 minute timeout.

## AI engines

- **What they see**: sender names, addresses, message counts and a few subjects, never message bodies.
- **Where it goes**: Claude Code and Ollama run on your computer (Ollama keeps everything local); Claude API, Groq and other OpenAI compatible services receive that data over https.
- **Claude Code** runs with no tools, no MCP servers, no user settings and no saved session, so it cannot read files or act.
- **Addresses**: a custom engine address must be https, or plain http only for `localhost`, `127.0.0.1` or `[::1]`. Model names are checked so they can never become command line flags.
- **Answers** must be JSON and are validated before use: only senders that were shown (or their domains, never public ones like gmail.com), sanitized label names, one label per sender.
- A malicious subject can at most change a proposal, which you review before anything is applied.

## Unsubscribe

- Ordo reads the `List-Unsubscribe` header of the sender's latest messages straight from Gmail; nothing from the interface is used as a link.
- One click unsubscribe (RFC 8058) sends a single POST, without following redirects, and only to public https hosts: no IP addresses, `localhost` or `.local` names.
- Other links open in your browser and `mailto:` addresses in your mail app, so you see and confirm them yourself.

## Updates

- Ordo checks `latest.json` from the GitHub releases on launch and installs only after you click.
- Every installer is signed with the project's updater key and the signature is verified before installing.

## Destructive actions

- Everything goes to the Gmail trash (recoverable for 30 days).
- Protected labels are excluded from every clean up query, including trashing a sender's mail.
- Emptying, deleting, blocking and moving to trash ask for a second click.
- Input from the UI is checked again in the engine (label names, senders, ids, models, addresses) and invalid input stops the operation before Gmail or the AI engine is touched.

## Desktop hardening

- Strict Content Security Policy.
- Minimal Tauri capabilities; links can only open Google, GitHub and the AI provider sites.

## Known limits

- Builds are not code signed for Windows yet, so SmartScreen may warn on first launch.
- With the Google OAuth app in testing mode the permission expires every 7 days.

## Reporting a vulnerability

Please open a private security advisory on the GitHub repository instead of a public issue.