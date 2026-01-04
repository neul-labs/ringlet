# Grok CLI

> Use MiniMax-M2.1 for AI programming in Grok CLI.

<Note>
  This tool is not recommended for Agent workflows. For better results, we suggest using **Claude Code** or **Cursor**.
</Note>

## Install Grok CLI

Use npm to install the Grok CLI globally:

```bash  theme={null}
npm install -g @vibe-kit/grok-cli
```

## Configure MiniMax API

<Warning>
  **Important: Clear OpenAI Environment Variables Before Configuration**

  Before configuring, ensure you clear the following OpenAI-related environment variables to avoid conflicts with MiniMax API:

  * `OPENAI_API_KEY`
  * `OPENAI_BASE_URL`
</Warning>

1. Set the base URL and API key using environment variables.

* The `GROK_BASE_URL` should be set based on your location: for international users, use `https://api.minimax.io/v1`; for users in China, use `https://api.minimaxi.com/v1`.
* Use the API key obtained from the [**MiniMax Developer Platform**](https://platform.minimax.io/user-center/basic-information/interface-key) (For users in China, visit [MiniMax Developer Platform](https://platform.minimaxi.com/user-center/basic-information/interface-key)) as the value for `MINIMAX_API_KEY`.

```bash  theme={null}
export GROK_BASE_URL="https://api.minimax.io/v1"  
export GROK_API_KEY="<MINIMAX_API_KEY>"
```

2. Start the Grok CLI with a specified model: MiniMax-M2.1

```bash  theme={null}
grok --model MiniMax-M2.1
```


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://platform.minimax.io/docs/llms.txt
