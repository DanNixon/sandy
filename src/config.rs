use serde::Deserialize;
use std::collections::BTreeMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub(crate) trait ToBwrapArgs {
    type Args = Vec<String>;
    fn bwrap_args(&self) -> Self::Args;
}

#[derive(Debug, Deserialize)]
pub(crate) struct Configuration {
    default_profile: Option<String>,
    profiles: Vec<Profile>,
}

impl Configuration {
    fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    fn get_default_profile(&self) -> Option<&Profile> {
        match &self.default_profile {
            Some(default_profile) => self.get_profile(default_profile),
            None => None,
        }
    }

    pub(crate) fn get_profile_for_program(&self, program: &str) -> Option<&Profile> {
        match self
            .profiles
            .iter()
            .find(|p| p.programs.contains(&program.to_string()))
        {
            Some(profile) => Some(profile),
            None => self.get_default_profile(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Profile {
    name: String,

    #[serde(default)]
    programs: Vec<String>,

    pub(crate) config: SandboxConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SandboxConfig {
    #[serde(default)]
    shared_namespaces: Vec<Namespace>,

    uid: Option<u32>,
    gid: Option<u32>,

    #[serde(default)]
    env: Environment,

    #[serde(default)]
    binds: Vec<Bind>,
    #[serde(default)]
    symlinks: Vec<Link>,
    #[serde(default)]
    dirs: Vec<Directory>,

    pub(crate) users: Option<Vec<String>>,
    pub(crate) groups: Option<Vec<String>>,
}

impl ToBwrapArgs for SandboxConfig {
    fn bwrap_args(&self) -> Self::Args {
        let mut args = Self::Args::new();

        if self.shared_namespaces.is_empty() {
            args.push("--unshare-all".to_string());
        } else {
            for ns in Namespace::iter() {
                if !self.shared_namespaces.contains(&ns) {
                    args.push(
                        match ns {
                            Namespace::User => "--unshare-user",
                            Namespace::Ipc => "--unshare-ipc",
                            Namespace::Pid => "--unshare-pid",
                            Namespace::Network => "--unshare-net",
                            Namespace::Uts => "--unshare-uts",
                            Namespace::CGroup => "--unshare-cgroup",
                        }
                        .to_string(),
                    );
                }
            }
        }

        if let Some(uid) = &self.uid {
            args.push("--uid".to_string());
            args.push(uid.to_string());
        }

        if let Some(gid) = &self.gid {
            args.push("--gid".to_string());
            args.push(gid.to_string());
        }

        args.append(&mut self.env.bwrap_args());

        args.append(
            &mut self
                .binds
                .iter()
                .map(|d| d.bwrap_args())
                .collect::<Vec<Vec<String>>>()
                .concat(),
        );

        args.append(
            &mut self
                .symlinks
                .iter()
                .map(|d| d.bwrap_args())
                .collect::<Vec<Vec<String>>>()
                .concat(),
        );

        args.append(
            &mut self
                .dirs
                .iter()
                .map(|d| d.bwrap_args())
                .collect::<Vec<Vec<String>>>()
                .concat(),
        );

        args
    }
}

#[derive(Debug, Deserialize, EnumIter, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Namespace {
    User,
    Ipc,
    Pid,
    Network,
    Uts,
    CGroup,
}

#[derive(Debug, Default, Deserialize)]
struct Environment {
    #[serde(default)]
    unset_all: bool,

    #[serde(default)]
    unset: Vec<String>,

    #[serde(default)]
    vars: BTreeMap<String, String>,
}

impl ToBwrapArgs for Environment {
    fn bwrap_args(&self) -> Self::Args {
        let mut args = Self::Args::new();

        if self.unset_all {
            args.push("--clearenv".to_string());
        }

        for k in self.unset.iter() {
            args.push("--unsetenv".to_string());
            args.push(k.to_string());
        }

        for (k, v) in self.vars.iter() {
            args.push("--setenv".to_string());
            args.push(k.to_string());
            args.push(v.to_string());
        }

        args
    }
}

#[derive(Debug, Deserialize)]
struct Bind {
    kind: BindKind,
    src: String,
    dest: String,
}

impl ToBwrapArgs for Bind {
    fn bwrap_args(&self) -> Self::Args {
        [
            match self.kind {
                BindKind::ReadWrite => "--bind",
                BindKind::ReadOnly => "--ro-bind",
                BindKind::Device => "--dev-bind",
            }
            .to_string(),
            self.src.to_string(),
            self.dest.to_string(),
        ]
        .to_vec()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BindKind {
    ReadWrite,
    ReadOnly,
    Device,
}

#[derive(Debug, Deserialize)]
struct Link {
    src: String,
    dest: String,
}

impl ToBwrapArgs for Link {
    fn bwrap_args(&self) -> Self::Args {
        [
            "--symlink".to_string(),
            self.src.to_string(),
            self.dest.to_string(),
        ]
        .to_vec()
    }
}

#[derive(Debug, Deserialize)]
struct Directory {
    kind: DirectoryKind,
    path: String,
}

impl ToBwrapArgs for Directory {
    fn bwrap_args(&self) -> Self::Args {
        [
            match self.kind {
                DirectoryKind::Directory => "--dir",
                DirectoryKind::Proc => "--proc",
                DirectoryKind::Dev => "--dev",
                DirectoryKind::TmpFs => "--tmpfs",
            }
            .to_string(),
            self.path.to_string(),
        ]
        .to_vec()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DirectoryKind {
    Directory,
    Proc,
    Dev,
    TmpFs,
}
