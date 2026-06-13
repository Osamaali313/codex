use std::fs;
use std::path::Path;

use codex_desktop_distribution::DesktopResources;
use codex_plugin::PluginId;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde_json::json;
use tempfile::TempDir;

use super::*;

struct Fixture {
    _temp: TempDir,
    resources: DesktopResources,
    plugin_data_root: AbsolutePathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("temp dir");
    let resources_root = temp.path().join("resources");
    write(
        &resources_root.join(BUNDLED_MARKETPLACE_PATH),
        &json!({
            "name": "openai-bundled",
            "plugins": [{
                "name": "computer-use",
                "source": {"source": "local", "path": "./plugins/computer-use"}
            }]
        })
        .to_string(),
    );

    let plugin_data_root = temp.path().join("data/computer-use");
    fs::create_dir_all(&plugin_data_root).expect("plugin data root");

    Fixture {
        resources: DesktopResources::from_trusted_path(resources_root).expect("Desktop resources"),
        plugin_data_root: AbsolutePathBuf::try_from(plugin_data_root)
            .expect("absolute plugin data root"),
        _temp: temp,
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("file parent")).expect("create parent");
    fs::write(path, contents).expect("write fixture");
}

fn replace_marketplace(fixture: &Fixture, marketplace: serde_json::Value) {
    write(
        &fixture.resources.root().join(BUNDLED_MARKETPLACE_PATH),
        &marketplace.to_string(),
    );
}

fn load(fixture: &Fixture) -> Result<Vec<PluginHookSource>, String> {
    let plugin_id = PluginId::parse("computer-use@openai-bundled").expect("plugin id");
    load_app_bundled_internal_hooks_from_resources(
        &fixture.resources,
        &plugin_id,
        &fixture.plugin_data_root,
    )
}

#[test]
fn marketplace_identity_and_plugin_source_must_match() {
    let cases = [
        (
            "wrong marketplace",
            json!({"name": "spoofed", "plugins": []}),
        ),
        (
            "wrong source",
            json!({
                "name": "openai-bundled",
                "plugins": [{
                    "name": "computer-use",
                    "source": {"source": "local", "path": "./plugins/other"}
                }]
            }),
        ),
    ];

    for (label, marketplace) in cases {
        let fixture = fixture();
        replace_marketplace(&fixture, marketplace);
        load(&fixture).expect_err(label);
    }
}

#[test]
fn non_bundled_marketplace_cannot_request_internal_hook_loading() {
    let fixture = fixture();
    let plugin_id = PluginId::parse("computer-use@spoofed").expect("plugin id");

    load_app_bundled_internal_hooks_from_resources(
        &fixture.resources,
        &plugin_id,
        &fixture.plugin_data_root,
    )
    .expect_err("wrong marketplace must be rejected");
}
