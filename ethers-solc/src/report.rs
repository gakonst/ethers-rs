//! Subscribe to events in the compiler pipeline

use crate::{CompilerInput, CompilerOutput, Solc};
use semver::Version;
use std::{
    error::Error,
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// Install this `Reporter` as the global default if one is
/// not already set.
///
/// # Errors
/// Returns an Error if the initialization was unsuccessful, likely
/// because a global reporter was already installed by another
/// call to `try_init`.
pub fn try_init<T>(reporter: T) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    T: Reporter + Send + Sync + 'static,
{
    set_global_reporter(Report::new(reporter))?;
    Ok(())
}

/// Install this `Reporter` as the global default.
///
/// # Panics
///
/// Panics if the initialization was unsuccessful, likely because a
/// global reporter was already installed by another call to `try_init`.
/// ```rust
/// use ethers_solc::report::BasicStdoutReporter;
/// let subscriber = ethers_solc::report::init(BasicStdoutReporter::default());
/// ```
pub fn init<T>(reporter: T)
where
    T: Reporter + Send + Sync + 'static,
{
    try_init(reporter).expect("Failed to install global reporter")
}

/// Trait representing the functions required to emit information about various steps in the
/// compiler pipeline.
///
/// This trait provides a series of callbacks that are invoked at certain parts of the
/// [`crate::Project::compile()`] process.
///
/// Implementers of this trait can use these callbacks to emit additional information, for example
/// print custom messages to `stdout`.
///
/// A `Reporter` is entirely passive and only listens to incoming "events".
pub trait Reporter: 'static {
    /// Callback invoked right before [`Solc::compile()`] is called
    fn on_solc_spawn(&self, _solc: &Solc, _version: &Version, _input: &CompilerInput) {}

    /// Invoked with the `CompilerOutput` if [`Solc::compiled()`] was successful
    fn on_solc_success(&self, _solc: &Solc, _version: &Version, _output: &CompilerOutput) {}

    /// Invoked before a new [`Solc`] bin is installed
    fn on_solc_installation_start(&self, _version: &Version) {}

    /// Invoked before a new [`Solc`] bin was successfully installed
    fn on_solc_installation_success(&self, _version: &Version) {}
}

pub(crate) fn solc_spawn(solc: &Solc, version: &Version, input: &CompilerInput) {
    with_global(|r| r.reporter.on_solc_spawn(solc, version, input));
}

pub(crate) fn solc_success(solc: &Solc, version: &Version, output: &CompilerOutput) {
    with_global(|r| r.reporter.on_solc_success(solc, version, output));
}

pub(crate) fn solc_installation_start(version: &Version) {
    with_global(|r| r.reporter.on_solc_installation_start(version));
}

pub(crate) fn solc_installation_success(version: &Version) {
    with_global(|r| r.reporter.on_solc_installation_success(version));
}

fn get_global() -> Option<&'static Report> {
    if GLOBAL_REPORTER_STATE.load(Ordering::SeqCst) != SET {
        return None
    }
    unsafe {
        // This is safe given the invariant that setting the global reporter
        // also sets `GLOBAL_REPORTER_STATE` to `SET`.
        Some(GLOBAL_REPORTER.as_ref().expect(
            "Reporter invariant violated: GLOBAL_REPORTER must be initialized before GLOBAL_REPORTER_STATE is set",
        ))
    }
}

/// Executes a closure with a reference to the `Reporter`.
pub fn with_global<T>(f: impl FnOnce(&Report) -> T) -> Option<T> {
    let dispatch = get_global()?;
    Some(f(dispatch))
}

/// A no-op [`Reporter`] that does nothing.
#[derive(Copy, Clone, Debug, Default)]
pub struct NoReporter(());

impl Reporter for NoReporter {}

/// A [`Reporter`] that emits some general information to `stdout`
#[derive(Copy, Clone, Debug, Default)]
pub struct BasicStdoutReporter(());

impl Reporter for BasicStdoutReporter {
    /// Callback invoked right before [`Solc::compile()`] is called
    fn on_solc_spawn(&self, _solc: &Solc, version: &Version, input: &CompilerInput) {
        println!(
            "Compiling {} files with {}.{}.{}",
            input.sources.len(),
            version.major,
            version.minor,
            version.patch
        );
    }

    /// Invoked with the `CompilerOutput` if [`Solc::compiled()`] was successful
    fn on_solc_success(&self, _: &Solc, _: &Version, _: &CompilerOutput) {
        println!("Compilation finished successfully");
    }
    /// Invoked before a new [`Solc`] bin is installed
    fn on_solc_installation_start(&self, version: &Version) {
        println!("installing solc version \"{}\"", version);
    }

    /// Invoked before a new [`Solc`] bin was successfully installed
    fn on_solc_installation_success(&self, version: &Version) {
        println!("Successfully installed solc {}", version);
    }
}

/// Returned if setting the global reporter fails.
#[derive(Debug)]
pub struct SetGlobalReporterError {
    // private marker so this type can't be initiated
    _priv: (),
}

impl fmt::Display for SetGlobalReporterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("a global reporter has already been set")
    }
}

impl Error for SetGlobalReporterError {}

/// `Report` trace data to a [`Reporter`].
pub struct Report {
    reporter: Arc<dyn Reporter + Send + Sync>,
}

impl Report {
    /// Returns a new `Report` that does nothing
    pub fn none() -> Self {
        Report { reporter: Arc::new(NoReporter::default()) }
    }

    /// Returns a `Report` that forwards to the given [`Reporter`].
    ///
    /// [`Reporter`]: ../reporter/trait.Reporter.html
    pub fn new<S>(reporter: S) -> Self
    where
        S: Reporter + Send + Sync + 'static,
    {
        Self { reporter: Arc::new(reporter) }
    }
}

// tracks the state of `GLOBAL_REPORTER`
static GLOBAL_REPORTER_STATE: AtomicUsize = AtomicUsize::new(UN_SET);

const UN_SET: usize = 0;
const SETTING: usize = 1;
const SET: usize = 2;

static mut GLOBAL_REPORTER: Option<Report> = None;

/// Sets this report as the global default for the duration of the entire program.
///
/// The global reporter can only be set once; additional attempts to set the global reporter will
/// fail. Returns `Err` if the global reporter has already been set.
fn set_global_reporter(report: Report) -> Result<(), SetGlobalReporterError> {
    // `compare_exchange` tries to store `SETTING` if the current value is `UN_SET`
    // this returns `Ok(_)` if the current value of `GLOBAL_REPORTER_STATE` was `UN_SET` and
    // `SETTING` was written, this guarantees the value is `SETTING`.
    if GLOBAL_REPORTER_STATE
        .compare_exchange(UN_SET, SETTING, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        unsafe {
            GLOBAL_REPORTER = Some(report);
        }
        GLOBAL_REPORTER_STATE.store(SET, Ordering::SeqCst);
        Ok(())
    } else {
        Err(SetGlobalReporterError { _priv: () })
    }
}
