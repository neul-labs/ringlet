# Droid

> Use MiniMax-M2.1 for AI programming in Droid.

## Install Droid

For Mac/Linux Users：

```bash  theme={null}
curl -fsSL https://app.factory.ai/cli | sh
```

For Windows Users:

```bash  theme={null}
irm https://app.factory.ai/cli/windows | iex
```

For more information, please refer to the [Droid documentation](https://docs.factory.ai/cli/getting-started/quickstart).

## Configure MiniMax API

<Warning>
  **Important: Clear Anthropic Environment Variables Before Configuration**

  Before configuring, ensure you clear the following Anthropic-related environment variables to avoid conflicts with MiniMax API:

  * `ANTHROPIC_AUTH_TOKEN`
  * `ANTHROPIC_BASE_URL`
</Warning>

1. Use following command to edit the configuration file located at `~/.factory/config.json`.

* The `base_url` should be set based on your location: for international users, use `https://api.minimax.io/anthropic`; for users in China, use `https://api.minimaxi.com/anthropic`.
* Set `<MINIMAX_API_KEY>` to the API key obtained from the [MiniMax Developer Platform](https://platform.minimax.io/user-center/basic-information/interface-key) (For users in China, visit [MiniMax Developer Platform](https://platform.minimaxi.com/user-center/basic-information/interface-key)).

<Note>
  Tips:

  * Config `~/.factory/config.json`, NOT `~/.factory/settings.json`
  * Clear the `ANTHROPIC_AUTH_TOKEN` environment variable, otherwise it will override the API key in `.factory/config.json` and cause errors
</Note>

```json  theme={null}
{
    "custom_models": [
        {
            "model_display_name": "MiniMax-M2.1",
            "model": "MiniMax-M2.1",
            "base_url": "https://api.minimax.io/anthropic",
            "api_key": "<MINIMAX_API_KEY>",
            "provider": "anthropic",
            "max_tokens": 64000
        }
    ]
}
```

2. Navigate to your project and start interactive session

```bash  theme={null}
cd /path/to/your/project
droid
```

3. Enter `/model`, select the "**MiniMax-M2.1**" model and use it in Droid.

<img src="https://filecdn.minimax.chat/public/6ff7915e-d9bf-47a5-b008-8d5fdd2d27e7.png" width="60%" />


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://platform.minimax.io/docs/llms.txt
