# Claude Code

> Use MiniMax-M2.1 for AI programming in Claude Code.

## Install Claude Code

Refer to the [Claude Code documentation](https://docs.claude.com/en/docs/claude-code/setup) for installation.

## Configure MiniMax API

<Warning>
  **Important: Clear Anthropic Environment Variables Before Configuration**

  Before configuring, ensure you clear the following Anthropic-related environment variables to avoid conflicts with MiniMax API:

  * `ANTHROPIC_AUTH_TOKEN`
  * `ANTHROPIC_BASE_URL`
</Warning>

1. Edit or create the Claude Code configuration file located at `~/.claude/settings.json`. In this file, add or update the `env` field as shown below.

* The `ANTHROPIC_BASE_URL` should be set based on your location: for international users, use `https://api.minimax.io/anthropic`; for users in China, use `https://api.minimaxi.com/anthropic`.
* Set `<MINIMAX_API_KEY>` to the API key obtained from the [MiniMax Developer Platform](https://platform.minimax.io/user-center/basic-information/interface-key) (For users in China, visit [MiniMax Developer Platform](https://platform.minimaxi.com/user-center/basic-information/interface-key)).
* Note: Environment variables `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` take priority over `settings.json` configuration.

```json  theme={null}
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.minimax.io/anthropic",
    "ANTHROPIC_AUTH_TOKEN": "<MINIMAX_API_KEY>",
    "API_TIMEOUT_MS": "3000000",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1,
    "ANTHROPIC_MODEL": "MiniMax-M2.1",
    "ANTHROPIC_SMALL_FAST_MODEL": "MiniMax-M2.1",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "MiniMax-M2.1",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "MiniMax-M2.1",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "MiniMax-M2.1"
  }
}
```

2. After completing the configuration, navigate to your working directory and run the `claude` command in the terminal to start using Claude Code. After startup, select **Trust This Folder** to allow it to access the files in your folder as shown below:

![claude-trust](https://filecdn.minimax.chat/public/ed3b7564-a187-4807-9ae8-218c50182103.PNG)

3. You can now start using Claude Code for development.

## Use M2.1 in Claude Code Extension for VS Code

<Steps>
  <Step title="Install Plugin">
    Install Claude Code Extension for VS Code

    <img src="https://filecdn.minimax.chat/public/6939e914-b090-4f4f-9c0b-1e394828c23c.jpg" width="80%" />
  </Step>

  <Step title="Open Settings">
    After installation, click **Settings**

    ![](https://filecdn.minimax.chat/public/d538a295-18e1-4381-ab35-3cfd2fbb7cfc.png)
  </Step>

  <Step title="Configure Model">
    Configure the model to `MiniMax-M2.1`

    * In Settings → `Claude Code: Selected Model`, enter `MiniMax-M2.1`

    ![](https://filecdn.minimax.chat/public/69297d7e-51ee-474b-9ff9-2eb788611972.png)

    Or

    * Click `Edit in settings.json`, modify `claude-code.selectedModel` to `MiniMax-M2.1` in the configuration file.

    <img src="https://filecdn.minimax.chat/public/5c11d0a2-a0b7-4677-ab67-1a3c95d3e35c.png" width="80%" />
  </Step>

  <Step title="Configure Environment Variables">
    * If Claude Code is already installed, please refer to the configuration above for environment variable settings.
    * If Claude Code is not installed, click `Edit in settings.json`

    ![](https://filecdn.minimax.chat/public/69297d7e-51ee-474b-9ff9-2eb788611972.png)

    Modify `claudeCode.environmentVariables` to following settings:

    * The `ANTHROPIC_BASE_URL` value should be set based on your location: for international users, use `https://api.minimax.io/anthropic`; for users in China, use `https://api.minimaxi.com/anthropic`

    ```json  theme={null}
    "claudeCode.environmentVariables": [
      {
        "name": "ANTHROPIC_BASE_URL",
        "value": "https://api.minimax.io/anthropic"
      },
      {
        "name": "ANTHROPIC_AUTH_TOKEN",
        "value": "<MINIMAX_API_KEY>"
      },
      {
        "name": "API_TIMEOUT_MS",
        "value": "3000000"
      },
      {
        "name": "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
        "value": "1"
      },
      {
        "name": "ANTHROPIC_MODEL",
        "value": "MiniMax-M2.1"
      },
      {
        "name": "ANTHROPIC_SMALL_FAST_MODEL",
        "value": "MiniMax-M2.1"
      },
      {
        "name": "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "value": "MiniMax-M2.1"
      },
      {
        "name": "ANTHROPIC_DEFAULT_OPUS_MODEL",
        "value": "MiniMax-M2.1"
      },
      {
        "name": "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "value": "MiniMax-M2.1"
      }
    ],
    ```
  </Step>
</Steps>


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://platform.minimax.io/docs/llms.txt
