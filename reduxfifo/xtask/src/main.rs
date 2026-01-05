use std::path::Path;

use clap::Parser as _;
use maven_utils::{Target, build_maven_zip, locate_roborio_toolchain};

use crate::maven_utils::{BuildConfig, locate_systemcore_toolchain};

pub mod maven_utils;

const GROUP_ID: &str = "com.reduxrobotics.frc";
const ARTIFACT_ID: &str = "ReduxLib-fifo";

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(num_args = 1..)]
    targets: Vec<Compileable>,
    #[arg(long = "static", help = "Compile static instead of shared binaries")]
    static_build: bool,
    #[arg(long = "debug", help = "Compile with debug symbols")]
    debug_build: bool,
    #[arg(
        long = "all",
        help = "Compile all permutations of shared/static and release/debug binaries"
    )]
    all_build: bool,
    #[arg(
        last = true,
        num_args = 1..,
        help = "args to pass through to Cargo"
    )]
    cargo_flags: Vec<String>,
}

impl Cli {
    fn build_configs(&self) -> Vec<BuildConfig> {
        if self.all_build {
            return vec![
                BuildConfig::Shared,
                BuildConfig::Static,
                BuildConfig::SharedDebug,
                BuildConfig::StaticDebug,
            ];
        }
        vec![match (self.static_build, self.debug_build) {
            (false, false) => BuildConfig::Shared,
            (false, true) => BuildConfig::SharedDebug,
            (true, false) => BuildConfig::Static,
            (true, true) => BuildConfig::StaticDebug,
        }]
    }
}

#[derive(clap::ValueEnum, Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Compileable {
    #[value(name = "linuxathena")]
    LinuxAthena,
    #[value(name = "linuxsystemcore")]
    LinuxSystemCore,
    #[value(name = "linuxx86-64")]
    LinuxX86_64,
    #[value(name = "linuxarm64")]
    LinuxArm64,
    #[value(name = "windowsx86-64")]
    WindowsX86_64,
    #[value(name = "windowsarm64")]
    WindowsArm64,
    #[value(name = "osxuniversal")]
    OsxUniversal,
    #[value(name = "headers")]
    Headers,
    #[default]
    #[value(name = "auto")]
    Auto,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    let build_configs = cli.build_configs();
    let cargo_flags = cli.cargo_flags;
    for target in cli.targets {
        match target {
            Compileable::LinuxAthena => {
                build_maven(Target::LinuxAthena, &build_configs, &cargo_flags)?
            }
            Compileable::LinuxX86_64 => {
                build_maven(Target::LinuxX86_64, &build_configs, &cargo_flags)?
            }
            Compileable::LinuxArm64 => {
                build_maven(Target::LinuxArm64, &build_configs, &cargo_flags)?
            }
            Compileable::LinuxSystemCore => {
                build_maven(Target::LinuxSystemCore, &build_configs, &cargo_flags)?
            }
            Compileable::WindowsX86_64 => {
                build_maven(Target::WindowsX86_64, &build_configs, &cargo_flags)?
            }
            Compileable::WindowsArm64 => {
                build_maven(Target::WindowsArm64, &build_configs, &cargo_flags)?
            }
            Compileable::OsxUniversal => {
                build_maven(Target::OsxUniversal, &build_configs, &cargo_flags)?
            }
            Compileable::Headers => {
                build_maven_zip(Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
            }
            Compileable::Auto => {
                // always build headers
                build_maven_zip(Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
                // always build linuxathena if possible
                if locate_roborio_toolchain().is_some() {
                    build_maven(Target::LinuxAthena, &build_configs, &cargo_flags)?;
                }

                if locate_systemcore_toolchain().is_some() {
                    build_maven(Target::LinuxSystemCore, &build_configs, &cargo_flags)?;
                }

                // build platform-dependent targets
                #[cfg(target_os = "linux")]
                {
                    build_maven(Target::LinuxX86_64, &build_configs, &cargo_flags)?;
                    if Path::new("/usr/local/aarch64-linux-gnu").exists() {
                        build_maven(Target::LinuxArm64, &build_configs, &cargo_flags)?;
                    }
                }

                #[cfg(target_os = "macos")]
                build_maven(Target::OsxUniversal, &build_configs, &cargo_flags)?;

                #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
                build_maven(Target::WindowsX86_64, &build_configs, &cargo_flags)?;

                #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
                build_maven(Target::WindowsArm64, &build_configs, &cargo_flags)?;
            }
        }
    }
    Ok(())
}

fn build_maven(
    target: Target,
    build_configs: &[BuildConfig],
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    maven_utils::build_maven(target, GROUP_ID, ARTIFACT_ID, build_configs, cargo_flags)?;
    maven_utils::build_maven_pom(GROUP_ID, ARTIFACT_ID)?;
    maven_utils::build_maven_metadata(GROUP_ID, ARTIFACT_ID)?;
    Ok(())
}
