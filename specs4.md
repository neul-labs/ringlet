# OpenCode

> Use MiniMax-M2.1 for AI programming in OpenCode.

## Install OpenCode

* Use `curl` to install the OpenCode

```bash  theme={null}
curl -fsSL https://opencode.ai/install | bash
```

* Use `npm` to install the OpenCode

```bash  theme={null}
npm i -g opencode-ai
```

For more information, please refer to the [OpenCode website](https://opencode.ai/)

## Configure MiniMax API

<Warning>
  **Important: Clear Anthropic Environment Variables Before Configuration**

  Before configuring, ensure you clear the following Anthropic-related environment variables to avoid conflicts with MiniMax API:

  * `ANTHROPIC_AUTH_TOKEN`
  * `ANTHROPIC_BASE_URL`
</Warning>

1. Edit or create the OpenCode configuration file located at `~/.config/opencode/opencode.json`. In this file, add or update the env field as shown below.

* The `baseURL` should be set based on your location: for international users, use `https://api.minimax.io/anthropic/v1`; for users in China, use `https://api.minimaxi.com/anthropic/v1`.
* Set `<MINIMAX_API_KEY>` to the API key obtained from the [MiniMax Developer Platform](https://platform.minimax.io/user-center/basic-information/interface-key) (For users in China, visit [MiniMax Developer Platform](https://platform.minimaxi.com/user-center/basic-information/interface-key)).

```json  theme={null}
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "minimax": {
      "npm": "@ai-sdk/anthropic",
      "options": {
        "baseURL": "https://api.minimax.io/anthropic/v1",
        "apiKey": "<MINIMAX_API_KEY> (Optional)"
      },
      "models": {
        "MiniMax-M2.1": {
          "name": "MiniMax-M2.1"
        }
      }
    }
  }
}
```

**Alternative Authentication Method:**

If you prefer not to add the API key directly to the configuration file, you can authenticate using the interactive login command. Follow these steps:

i. Run the authentication command:

```bash  theme={null}
opencode auth login
```

ii. When prompted, select provider **"Other"**:

<img src="https://filecdn.minimax.chat/public/f616dac4-2ec4-4fc0-aabb-9776d5ec9043.png" width="60%" />

iii. Enter the provider ID as **"minimax"**:

<img src="https://filecdn.minimax.chat/public/4a7d2327-651d-450a-b041-dd17e5ac78ec.png" width="60%" />

iv. Enter your **MiniMax API key** when prompted:

<img src="https://filecdn.minimax.chat/public/74a8214a-8f4a-4019-b335-c9a9717db8da.png" width="80%" />

2. Navigate to your project and start interactive session

```bash  theme={null}
cd /path/to/your/project

opencode
```

3. Enter `/models`, select the "MiniMax-M2.1" model and use it in OpenCode

<img src="https://filecdn.minimax.chat/public/93ece1fb-5841-4056-9980-c872f0f5ebc0.png" width="80%" />


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://platform.minimax.io/docs/llms.txt
