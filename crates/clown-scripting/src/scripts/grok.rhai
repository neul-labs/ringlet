// Grok CLI configuration script
// Generates config files for Grok CLI

// Grok uses OpenAI-compatible API
let config = #{
    model: ctx.profile.model,
    api_base: ctx.profile.endpoint
};

// Build environment variables
let env = #{};
env[ctx.provider.auth_env_key] = "${API_KEY}";  // Placeholder, replaced at runtime
env["OPENAI_API_BASE"] = ctx.profile.endpoint;

// Return the output
#{
    files: #{
        ".grok/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
