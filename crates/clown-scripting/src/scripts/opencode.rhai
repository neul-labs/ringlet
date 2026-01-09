// OpenCode configuration script
// Generates config files for OpenCode CLI agent

// OpenCode uses Anthropic-compatible API
let config = #{
    model: ctx.profile.model,
    provider: if ctx.provider.type == "anthropic" { "anthropic" } else { "anthropic-compatible" }
};

// If using a custom endpoint, set it
if ctx.provider.type != "anthropic" {
    config.api_base = ctx.profile.endpoint;
}

// Build environment variables
let env = #{};
env[ctx.provider.auth_env_key] = "${API_KEY}";  // Placeholder, replaced at runtime
if ctx.provider.type != "anthropic" {
    env["ANTHROPIC_BASE_URL"] = ctx.profile.endpoint;
}

// Return the output
#{
    files: #{
        ".opencode/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
