//! mocked project tests

use ethers_solc::{
    error::Result,
    project_util::{
        mock::{MockProjectGenerator, MockProjectSettings},
        TempProject,
    },
};

// default version to use
const DEFAULT_VERSION: &str = "^0.8.10";

struct MockSettings {
    settings: MockProjectSettings,
    version: &'static str,
}

impl From<MockProjectSettings> for MockSettings {
    fn from(settings: MockProjectSettings) -> Self {
        MockSettings { settings, version: DEFAULT_VERSION }
    }
}
impl From<(MockProjectSettings, &'static str)> for MockSettings {
    fn from(input: (MockProjectSettings, &'static str)) -> Self {
        MockSettings { settings: input.0, version: input.1 }
    }
}

/// Helper function to run a test and report the used generator if the closure failed.
fn run_mock(
    settings: impl Into<MockSettings>,
    f: impl FnOnce(&mut TempProject) -> Result<()>,
) -> TempProject {
    let MockSettings { settings, version } = settings.into();
    let gen = MockProjectGenerator::new(&settings);
    let mut project = TempProject::dapptools().unwrap();
    let remappings = gen.remappings_at(project.root());
    project.paths_mut().remappings.extend(remappings);
    project.mock(&gen, version).unwrap();

    if let Err(err) = f(&mut project) {
        panic!(
            "mock failed: `{}` with mock settings:\n {}",
            err,
            serde_json::to_string(&gen).unwrap()
        );
    }

    project
}

/// Runs a basic set of tests for the given settings
fn run_basic(settings: impl Into<MockSettings>) {
    let settings = settings.into();
    let version = settings.version;
    run_mock(settings, |project| {
        project.ensure_no_errors_recompile_unchanged()?;
        project.add_basic_source("Dummy", version)?;
        project.ensure_changed()?;
        Ok(())
    });
}

#[test]
fn can_compile_mocked_random() {
    run_basic(MockProjectSettings::random());
}

// compile a bunch of random projects
#[test]
fn can_compile_mocked_multi() {
    for _ in 0..10 {
        run_basic(MockProjectSettings::random());
    }
}

#[test]
fn can_compile_mocked_large() {
    run_basic(MockProjectSettings::large())
}
