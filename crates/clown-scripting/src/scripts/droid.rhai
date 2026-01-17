// Droid configuration script
// Generates ~/.factory/config.json for Droid CLI
// Note: Droid uses custom_models array with API key in config (not env vars)

let config = #{};
let env = #{};

// For self-auth, no config needed - Droid handles its own auth
// For custom providers, use custom_models array
if ctx.provider.type != "self" && ctx.provider.type != "anthropic" {
    // Custom provider (MiniMax, Z.AI, etc.) - use custom_models config
    config.custom_models = [
        #{
            model_display_name: ctx.profile.model,
            model: ctx.profile.model,
            base_url: ctx.profile.endpoint,
            api_key: "${API_KEY}",
            provider: "anthropic",
            max_tokens: 64000
        }
    ];
}

// Return the output
#{
    files: #{
        ".factory/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
