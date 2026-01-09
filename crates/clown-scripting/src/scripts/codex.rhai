// Codex CLI configuration script
// Generates config files for OpenAI Codex CLI

// Codex uses OpenAI API
let config = #{
    model: ctx.profile.model,
    provider: "openai"
};

// If using a custom endpoint, set it
if ctx.provider.type != "openai" {
    config.api_base = ctx.profile.endpoint;
}

// Build environment variables
let env = #{};

// Only set API key env var if not self-authenticating
if ctx.provider.type != "self" {
    env[ctx.provider.auth_env_key] = "${API_KEY}";  // Placeholder, replaced at runtime
    if ctx.provider.type != "openai" {
        env["OPENAI_API_BASE"] = ctx.profile.endpoint;
    }
}

// Return the output
#{
    files: #{
        ".codex/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
