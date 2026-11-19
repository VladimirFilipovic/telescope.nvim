//! Registers the telescope MCP server, which exposes telescope-style search to
//! Zed's agent. Zed's extension API has no picker, panel or command API, so a
//! context server is the only live interactive surface an extension can add.

use zed_extension_api::{self as zed, settings::ContextServerSettings, ContextServerConfiguration, ContextServerId, Project};

const DEFAULT_BINARY: &str = "telescope-zed-mcp";

struct TelescopeExtension;

impl zed::Extension for TelescopeExtension {
    fn new() -> Self {
        TelescopeExtension
    }

    fn context_server_command(&mut self, id: &ContextServerId, project: &Project) -> zed::Result<zed::Command> {
        let settings = ContextServerSettings::for_project(id.as_ref(), project)?;
        let command = settings.command.unwrap_or(zed::settings::CommandSettings { path: None, arguments: None, env: None });
        let mut env: Vec<(String, String)> = command.env.unwrap_or_default().into_iter().collect();
        if let Some(root) = settings.settings.as_ref().and_then(|s| s.get("root")).and_then(|r| r.as_str()) {
            env.push(("TELESCOPE_ROOT".into(), root.into()));
        }
        Ok(zed::Command {
            command: command.path.unwrap_or_else(|| DEFAULT_BINARY.into()),
            args: command.arguments.unwrap_or_default(),
            env,
        })
    }

    fn context_server_configuration(
        &mut self,
        _id: &ContextServerId,
        _project: &Project,
    ) -> zed::Result<Option<ContextServerConfiguration>> {
        Ok(Some(ContextServerConfiguration {
            installation_instructions: include_str!("../configuration/installation_instructions.md").into(),
            settings_schema: include_str!("../configuration/settings_schema.json").into(),
            default_settings: include_str!("../configuration/default_settings.jsonc").into(),
        }))
    }
}

zed::register_extension!(TelescopeExtension);
