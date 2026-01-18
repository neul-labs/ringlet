// Codex CLI configuration script
// Generates ~/.codex/config.toml for OpenAI Codex CLI

let env = #{};
let config = "";

if ctx.provider.type == "self" || ctx.provider.type == "openai" {
    // For native OpenAI or self-auth, minimal config
    // Codex will use its built-in OpenAI provider
    config = `# Codex CLI configuration (managed by clown)
`;
} else {
    // For openai-compatible providers (MiniMax, Z.AI, etc.)
    // Generate full config.toml with custom provider
    config = `# Codex CLI configuration (managed by clown)
model = "` + ctx.profile.model + `"
model_provider = "clown"

[model_providers.clown]
name = "` + ctx.provider.name + `"
base_url = "` + ctx.profile.endpoint + `"
env_key = "` + ctx.provider.auth_env_key + `"
wire_api = "chat"
`;

    // Set API key env var
    env[ctx.provider.auth_env_key] = "${API_KEY}";
}

// Return the output
#{
    files: #{
        ".codex/config.toml": config
    },
    env: env,
    args: [],
    hooks: (),
    mcp_servers: ()
}
