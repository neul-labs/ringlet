// Codex CLI configuration script
// Configures OpenAI Codex CLI for custom providers via -c flags
// (Codex doesn't respect $HOME or XDG_CONFIG_HOME for config paths)

// Build environment variables
let env = #{};

// Build args for codex
let args = [];

if ctx.provider.type == "self" || ctx.provider.type == "openai" {
    // For native OpenAI or self-auth, no configuration needed
    // Codex will use its built-in OpenAI provider
} else {
    // For openai-compatible providers (like MiniMax), configure via -c flags
    // since codex doesn't respect $HOME for config path

    // Configure the custom model provider
    args.push("-c");
    args.push("model_providers.clown.name=\"" + ctx.provider.name + "\"");

    args.push("-c");
    args.push("model_providers.clown.base_url=\"" + ctx.profile.endpoint + "\"");

    args.push("-c");
    args.push("model_providers.clown.env_key=\"" + ctx.provider.auth_env_key + "\"");

    args.push("-c");
    args.push("model_providers.clown.wire_api=\"chat\"");

    args.push("-c");
    args.push("model_providers.clown.requires_openai_auth=false");

    // Set the model and provider to use
    args.push("-c");
    args.push("model=\"" + ctx.profile.model + "\"");

    args.push("-c");
    args.push("model_provider=\"clown\"");

    // Set API key env var
    env[ctx.provider.auth_env_key] = "${API_KEY}";
}

// Return the output - no files needed, using -c args instead
#{
    files: #{},
    env: env,
    args: args,
    hooks: (),
    mcp_servers: ()
}
