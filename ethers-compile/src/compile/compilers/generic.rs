use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use semver::Version;

use crate::{
    compile::CompilerTrait,
    error::{CompilerError, Result},
    version_from_output, CompilerInput, CompilerOutput, Solc, Vyper,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum CompilerKindEnum {
    Solc,
    Vyper,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct GenericCompiler {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub kind: Option<CompilerKindEnum>,
}

impl CompilerTrait for GenericCompiler {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn arg(&mut self, arg: String) {
        self.args.push(arg);
    }

    fn args(&mut self, args: Vec<String>) {
        for arg in args {
            self.arg(arg);
        }
    }

    fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    fn version(&self) -> Version {
        version_from_output(
            Command::new(&self.path)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .map_err(|err| CompilerError::io(err, &self.path))
                .expect("version"),
        )
        .expect("version")
    }

    fn language(&self) -> String {
        match &self.kind {
            Some(compiler) => match compiler {
                CompilerKindEnum::Solc => Solc::compiler_language(),
                CompilerKindEnum::Vyper => Vyper::compiler_language(),
            },
            None => "generic".to_string(),
        }
    }

    fn compile_exact(&self, _input: &CompilerInput) -> Result<CompilerOutput> {
        Err(CompilerError::Message("generic compiler cannot compile".to_string()))
    }

    fn compile(&self, _input: &CompilerInput) -> Result<CompilerOutput> {
        Err(CompilerError::Message("generic compiler cannot compile".to_string()))
    }
}

impl GenericCompiler {
    pub fn set_kind(&self, kind: CompilerKindEnum) -> Self {
        Self { path: self.path(), args: self.get_args(), kind: Some(kind) }
    }

    pub fn into_kind(&self) -> Option<Box<dyn CompilerTrait>> {
        match &self.kind {
            Some(compiler) => match compiler {
                CompilerKindEnum::Solc => Some(Box::new(self.to_solc())),
                CompilerKindEnum::Vyper => Some(Box::new(self.to_vyper())),
            },
            None => todo!(),
        }
    }

    pub fn to_vyper(&self) -> Vyper {
        Vyper { args: self.get_args(), ..Default::default() }
    }

    pub fn to_solc(&self) -> Solc {
        Solc { solc: self.path(), args: self.get_args() }
    }
}

impl From<Solc> for GenericCompiler {
    fn from(solc: Solc) -> Self {
        GenericCompiler { path: solc.solc, args: solc.args, kind: Some(CompilerKindEnum::Solc) }
    }
}

impl From<Vyper> for GenericCompiler {
    fn from(vyper: Vyper) -> Self {
        GenericCompiler { path: vyper.vyper, args: vyper.args, kind: Some(CompilerKindEnum::Vyper) }
    }
}
