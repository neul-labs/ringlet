# Codex CLI

> Use MiniMax-M2.1 for AI programming in Codex CLI.

<Note>
  To facilitate integrating models with the Codex CLI, it is recommended to use the `codex-MiniMax-M2.1` model. While models can interact with files using tools like Bash, this approach is not considered optimal for Agent workflows. For better Agent-based practices, we suggest using platforms such as **Claude Code** or **Cursor**.
</Note>

## Install Codex CLI

Use `npm` to install the Codex CLI globally:

```bash  theme={null}
npm i -g @openai/codex
```

## Configure MiniMax API

<Warning>
  **Important: Clear OpenAI Environment Variables Before Configuration**

  Before configuring, ensure you clear the following OpenAI-related environment variables to avoid conflicts with MiniMax API:

  * `OPENAI_API_KEY`
  * `OPENAI_BASE_URL`
</Warning>

1. Add the following configuration to the `.codex/config.toml` file.

* The `base_url` should be set based on your location: for international users, use `https://api.minimax.io/v1`; for users in China, use `https://api.minimaxi.com/v1`

```toml  theme={null}
[model_providers.minimax]
name = "MiniMax Chat Completions API"
base_url = "https://api.minimax.io/v1"
env_key = "MINIMAX_API_KEY"
wire_api = "chat"
requires_openai_auth = false
request_max_retries = 4
stream_max_retries = 10
stream_idle_timeout_ms = 300000

[profiles.m21]
model = "codex-MiniMax-M2.1"
model_provider = "minimax"
```

2. Set the API key using environment variables in the current terminal session for security reasons. Use the API key obtained from the [**MiniMax Developer Platform**](https://platform.minimax.io/user-center/basic-information/interface-key) (For users in China, visit [MiniMax Developer Platform](https://platform.minimaxi.com/user-center/basic-information/interface-key)) as the value for `MINIMAX_API_KEY`.

```bash  theme={null}
export MINIMAX_API_KEY="<MINIMAX_API_KEY>"
```

3. Start the Codex CLI with the specified profile.

```bash  theme={null}
codex --profile m21
```


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://platform.minimax.io/docs/llms.txtq
