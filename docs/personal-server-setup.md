# Set up a personal server

A personal server is waymark as a remote MCP server for one person. Chat apps on a phone (Claude, ChatGPT, Gemini) and other MCP clients use your memory repository through one web address. The server keeps no database and no stored state. Git holds the memory.

This guide is written for an AI agent to follow, and a person can read it as a recipe. The spec is [specs/foundation/personal-server](../specs/foundation/personal-server/requirements.md).

> **Status:** The server is built and tested against a fake GitHub. It is **not yet tested against the real apps and a real GitHub App**. Steps and notes that depend on that are marked **to test in real use**. If a step fails, tell the maintainer what you saw.

<br/>

## Index

- [Rules for the agent](#rules-for-the-agent)
- [What you need](#what-you-need)
- [The settings](#the-settings)
- [Step 1. Create the memory repository (Person)](#step-1-create-the-memory-repository-person)
- [Step 2. Make the signing key (Person)](#step-2-make-the-signing-key-person)
- [Step 3. Create the GitHub App (Person)](#step-3-create-the-github-app-person)
- [Step 4. Install the App on the one repository (Person)](#step-4-install-the-app-on-the-one-repository-person)
- [Step 5. Deploy the image and set the secrets (Person or agent, with the person's host login)](#step-5-deploy-the-image-and-set-the-secrets-person-or-agent-with-the-persons-host-login)
- [Step 6. Update the callback address (Person)](#step-6-update-the-callback-address-person)
- [Step 7. Connect an app (Person)](#step-7-connect-an-app-person)
- [Step 8. Set up the memory repository](#step-8-set-up-the-memory-repository)
- [Step 9. Check](#step-9-check)
- [If something fails](#if-something-fails)
- [Sign out every app](#sign-out-every-app)
- [What the server logs](#what-the-server-logs)

<br/>

## Rules for the agent

- Do the steps in order. Do one step at a time.
- A step marked **Person** can only be done by the person: it needs their login, a click in a web page, or a secret. Stop there. Say what the person must do and wait.
- **Never print a secret in the chat.** This covers the signing key, the private key of the GitHub App, the client secret, and every token. Do not ask the person to paste one into the chat. A secret goes only into the secret store of the host.
- Ask the person one question at a time.

<br/>

## What you need

- A GitHub account that owns the memory repository.
- A host that runs a container image and shows you a public HTTPS address. It can run all the time or start for each request. The host provides the certificate. Waymark speaks plain HTTP behind it and listens on `$PORT` (default 8080).
- `git` 2.31 or newer on the host (the image has it).

<br/>

## The settings

Waymark reads every setting from an environment variable. A secret may also come from a mounted file: set `<NAME>_FILE` to the path of the file.

| Setting | Secret? | What it is |
|---|---|---|
| `WAYMARK_MODE` | no | `personal-server`. The image sets it. |
| `PORT` | no | The port to listen on. Default 8080. Many hosts set it. |
| `WAYMARK_PUBLIC_URL` | no | The public address of the server, for example `https://memory.example.org`. It must start with `https://`. |
| `WAYMARK_REPO` | no | The HTTPS clone URL of the memory repository, for example `https://github.com/you/memory.git`. |
| `WAYMARK_OWNER` | no | Your GitHub account name. Only this account can sign in. |
| `WAYMARK_SIGNING_KEY` | **yes** | At least 32 random characters. It signs all tokens. A new key signs out every app. |
| `WAYMARK_GITHUB_APP_ID` | no | The App ID of your GitHub App. |
| `WAYMARK_GITHUB_INSTALLATION_ID` | no | The installation number of the App on your repository. |
| `WAYMARK_GITHUB_APP_KEY` | **yes** | The private key of the GitHub App (PEM). Use `WAYMARK_GITHUB_APP_KEY_FILE` for a mounted file. |
| `WAYMARK_GITHUB_CLIENT_ID` | no | The client ID of the GitHub App. |
| `WAYMARK_GITHUB_CLIENT_SECRET` | **yes** | A client secret of the GitHub App. |
| `WAYMARK_ACCESS_TOKEN_SECONDS` | no | Optional. Lifetime of an access token. Default 3600. |
| `WAYMARK_RENEWAL_TOKEN_SECONDS` | no | Optional. Lifetime of a renewal token. Default 2592000 (30 days). |
| `WAYMARK_CLONE_WAIT_SECONDS` | no | Optional. How long the first call waits for the first clone. Default 60. |

The server refuses to start when a required setting is missing, and the message names it. Do not set `WAYMARK_AUTHOR_NAME`, `WAYMARK_AUTHOR_EMAIL`, or `--repo`: the author is the person who signs in, and the repository comes from `WAYMARK_REPO`.

Server limits (read-only access, paused writes, and so on) go into a config file that you mount and name with `WAYMARK_SYSTEM_CONFIG`. See [local-install.md](local-install.md) for the format.

<br/>

## Step 1. Create the memory repository (Person)

**Do:** On GitHub, create an **empty private** repository, for example `memory`. Do not add a README. Note its HTTPS clone URL.

**You should see:** An empty repository with no commit.

**If not:** A repository with files is fine too if it already is a memory repository. For a new one, keep it empty.

<br/>

## Step 2. Make the signing key (Person)

**Do:** Make a random key and keep it for step 5. Do not paste it into a chat.

```bash
openssl rand -base64 48
```

**You should see:** A line of 64 characters.

<br/>

## Step 3. Create the GitHub App (Person)

**Do:** On GitHub open Settings, Developer settings, GitHub Apps, New GitHub App. Set:

- Name: anything, for example `waymark-memory`.
- Homepage URL: the public address of your server (you may not know it yet: use any address and edit it later).
- Callback URL: `<WAYMARK_PUBLIC_URL>/callback` (edit it after step 5 if needed).
- Expire user authorization tokens: leave on. Request user authorization during installation: off.
- Webhook: switch **Active** off.
- Repository permissions: **Contents: Read and write**. (Metadata: Read is added for you.)
- Account permissions: **Email addresses: Read-only**. Waymark needs it to read the verified email for the commit author. **To test in real use:** GitHub documents this permission for fine-grained tokens. This guide assumes that it also applies to a GitHub App.
- Where can this App be installed: **Only on this account**.

Then create the App. On the App page:

1. Note the **App ID** and the **Client ID**.
2. Under Client secrets, **Generate a new client secret**. Copy it once.
3. Under Private keys, **Generate a private key**. A `.pem` file downloads.

**You should see:** An App ID, a client ID, a client secret, and a `.pem` file.

<br/>

## Step 4. Install the App on the one repository (Person)

**Do:** On the App page open Install App, choose your account, choose **Only select repositories**, and select the memory repository. After the install, look at the address of the page. It ends in `/installations/<number>`. That number is the installation number.

**You should see:** The memory repository, and no other, in the list of the installation.

<br/>

## Step 5. Deploy the image and set the secrets (Person or agent, with the person's host login)

**Do:** Build the image from this repository and run it on your host:

```bash
docker build -t waymark-personal-server .
```

Push it to the registry of your host, or let the host build it from the `Dockerfile`. Set the settings from the table above in the secret store of the host:

- Secrets (`WAYMARK_SIGNING_KEY`, `WAYMARK_GITHUB_APP_KEY`, `WAYMARK_GITHUB_CLIENT_SECRET`): from steps 2 and 3. Never put them in the image, in the repository, or in a chat.
- The other settings as plain values.

Set `WAYMARK_PUBLIC_URL` to the address that the host gives you. Make the health check of the host call `/health`.

**You should see:** The log of the host shows a line like `event=start mode="personal-server"`. Open the public address in a browser: a blue page with "Waymark" and "The server is running." appears.

**If not:** The log shows why the server stopped. It names the missing or wrong setting. A wrong private key stops the start with a message that does not print the key.

**Cold start:** On a host that starts the server for each request, the first call after a pause clones the repository again. A large repository can take longer than the wait of the AI app or of the host. Then the first call fails with "Call again in a minute", and the next call works. Pick a host that keeps the process running if this is a problem.

<br/>

## Step 6. Update the callback address (Person)

**Do:** If you used a placeholder in step 3, edit the Callback URL of the GitHub App to `<WAYMARK_PUBLIC_URL>/callback`.

<br/>

## Step 7. Connect an app (Person)

The address of the MCP server is `<WAYMARK_PUBLIC_URL>/mcp`. The first time an app connects, it opens GitHub to sign in. Sign in with the account in `WAYMARK_OWNER`. Any other account is refused.

Which method each app uses is **to test in real use**.

- **Claude** (web, desktop, phone): Settings, Connectors, Add custom connector. Enter the address. Claude identifies itself with a client document at its own address, or registers itself. No other input is needed.
- **ChatGPT:** Developer mode and custom MCP connectors are for some plans. OpenAI names Business and Enterprise/Edu on the web. Check the current OpenAI help page for your plan. Add the connector with the address and choose OAuth.
- **Gemini:** The consumer app is not documented for custom servers. Gemini Enterprise (an admin setup, in preview) and the `agy` CLI need a client ID and a client secret that you enter by hand. Get them from the signing key (see below).
- **Claude Code:** `claude mcp add --transport http waymark <WAYMARK_PUBLIC_URL>/mcp`, then sign in when the tool asks.
- **Codex CLI:** `codex mcp add waymark --url <WAYMARK_PUBLIC_URL>/mcp`, then `codex mcp login waymark`.
- **`agy` CLI:** add the server to `mcp_config.json` with `serverUrl` set to the address. If the CLI cannot register itself, add `oauth` with the client ID and the client secret (see below).

**Client ID and secret for an app that needs them by hand (Person).** Run this on your own computer with the same signing key as the server. Never run it in a chat:

```bash
WAYMARK_SIGNING_KEY='<the key from step 2>' waymark --print-client
```

It prints a client ID and a client secret that are derived from the key. Enter them in the app. The redirect address of the app must be an `https://` address.

<br/>

## Step 8. Set up the memory repository

**Do:** If the repository is empty, ask your agent in the connected app: "Set up my waymark memory repository and call it <name>." The agent calls `status`, which says that the repository is empty, and then `init`. `init` commits and pushes to the repository.

**You should see:** The first commit appears on GitHub, with your name and your verified email as the author.

<br/>

## Step 9. Check

1. Open `<WAYMARK_PUBLIC_URL>` in a browser. The status page opens.
2. In the connected app, ask: "Search my waymark memory for anything."
3. Ask the agent to save one document, then look at the history of the repository on GitHub. The commit author is you.

<br/>

## If something fails

| What you see | Likely cause |
|---|---|
| The server stops at start and names a setting | A secret or a setting is missing or wrong. Fix it in the secret store. |
| The sign-in page says "This server is not for this account." | You signed in with a GitHub account other than `WAYMARK_OWNER`. |
| The sign-in says "Add and verify an email address on your GitHub account." | The account has no verified email that is not a no-reply address. Add one in GitHub settings. Check that the App has the permission "Email addresses: Read-only". |
| A tool says "Call again in a minute." | The first clone is still running. Wait and call again. |
| A write fails with a sign-in error for Git | The App is not installed on the repository, or it has no Contents write permission. |
| An app asks you to sign in again and again | The signing key changed, or the app does not accept the renewal token. Tell the maintainer. **To test in real use.** |

<br/>

## Sign out every app

Change `WAYMARK_SIGNING_KEY` and restart the server. Every token stops working and every app signs in again. A single app cannot be signed out on its own.

<br/>

## What the server logs

One line for each start, sign-in, and tool call, on standard output: the time, the event, the tool, the result, and the document ID. The log never holds a secret, a token, or document text. The host shows it. The server stores no log.
