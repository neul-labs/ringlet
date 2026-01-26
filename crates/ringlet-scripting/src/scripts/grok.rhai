// Grok CLI configuration script
// Generates config files for Grok CLI

// Grok uses OpenAI-compatible API
let config = #{
    model: ctx.profile.model
};

// Build environment variables
let env = #{};

// Only set API key and endpoint env vars if not self-authenticating
// For self-auth, Grok CLI will use its built-in xAI provider
if ctx.provider.type != "self" {
    config.api_base = ctx.profile.endpoint;
    env["GROK_BASE_URL"] = ctx.profile.endpoint;
    env["GROK_API_KEY"] = "${API_KEY}";
}

// Return the output
#{
    files: #{
        ".grok/config.json": json::encode_pretty(config)
    },
    env: env,
    hooks: (),
    mcp_servers: ()
}
