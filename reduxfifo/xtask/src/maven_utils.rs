use sha1::Digest;
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use zip::write::SimpleFileOptions;

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
fn target_dir() -> PathBuf {
    project_root().join("target")
}

static CARGO_TOML_INFO: LazyLock<cargo_toml::Manifest> = LazyLock::new(|| {
    let cargo_toml_data = std::fs::read(project_root().join("Cargo.toml"))
        .expect("Could not open ReduxFIFO Cargo.toml");
    cargo_toml::Manifest::from_slice(cargo_toml_data.as_slice())
        .expect("Could not parse ReduxFIFO Cargo.toml")
});

pub fn version() -> String {
    CARGO_TOML_INFO
        .workspace
        .clone()
        .unwrap()
        .package
        .unwrap()
        .version
        .unwrap()
}

/// derive the year to get the version of the wpilib toolchain to use.
pub fn year() -> u64 {
    let version = semver::Version::parse(&version())
        .expect("ReduxFIFO Cargo.toml is not in 'year-semver' format");
    version.major
}

/// systemcore has a hardcoded year.
pub const SYSTEMCORE_YEAR: &str = "2027_alpha1";

pub fn locate_roborio_toolchain() -> Option<PathBuf> {
    let toolchain_name = format!("arm-frc{}-linux-gnueabi-gcc", 2025);
    if let Ok(w) = which::which(&toolchain_name) {
        // sometimes the roborio toolchain is already in PATH (e.g. in buildserver containers)
        return Some(w.parent().unwrap().into());
    }

    #[cfg(unix)]
    let search_path = &[
        // All unicies have their wpilib install in the home directory.
        homedir::my_home()
            .ok()??
            .join(format!("wpilib/{}/roborio/bin", year())),
        // the roborio-cross container
        PathBuf::from("/usr/local/bin"),
    ];

    #[cfg(windows)]
    let search_path = &[
        // windows typically puts the roborio toolchain in C:\Users\Public for whatever reason
        PathBuf::from(std::env::var("PUBLIC").unwrap_or("C:\\Users\\Public".into()))
            .join(format!("wpilib\\{}\\roborio\\bin", year())),
        homedir::my_home()
            .ok()??
            .join(format!("wpilib\\{}\\roborio\\bin", year())),
    ];

    for candidate in search_path {
        let gcc = candidate.join(&toolchain_name);
        if gcc.exists() && gcc.is_file() {
            return Some(candidate.clone());
        }
    }
    None
}

pub fn locate_systemcore_toolchain() -> Option<PathBuf> {
    if let Ok(w) = which::which("aarch64-bookworm-linux-gnu-gcc") {
        // sometimes the systemcore toolchain is already in PATH (e.g. in buildserver containers)
        return Some(w.parent().unwrap().into());
    }

    #[cfg(unix)]
    let search_path = &[
        // All unicies have their wpilib install in the home directory.
        homedir::my_home()
            .ok()??
            .join(format!("wpilib/{}/systemcore/bin", SYSTEMCORE_YEAR)),
        // the roborio-cross container
        PathBuf::from("/usr/local/bin"),
    ];

    #[cfg(windows)]
    let search_path = &[
        // windows typically puts the roborio toolchain in C:\Users\Public for whatever reason
        PathBuf::from(std::env::var("PUBLIC").unwrap_or("C:\\Users\\Public".into()))
            .join(format!("wpilib\\{}\\systemcore\\bin", SYSTEMCORE_YEAR)),
        homedir::my_home()
            .ok()??
            .join(format!("wpilib\\{}\\roborio\\bin", SYSTEMCORE_YEAR)),
    ];

    for candidate in search_path {
        let gcc = candidate.join("aarch64-bookworm-linux-gnu-gcc");
        if gcc.exists() && gcc.is_file() {
            return Some(candidate.clone());
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    LinuxAthena,
    LinuxSystemCore,
    WindowsX86_64,
    WindowsArm64,
    OsxUniversal,
    OsxX86_64,
    OsxArm64,
    LinuxX86_64,
    LinuxArm64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Osx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binary {
    Lib(&'static str),
    Win(&'static str),
    WinLib(&'static str),
}

impl Binary {
    pub fn source_name(&self, name: &str) -> String {
        match self {
            Binary::Lib(s) => format!("lib{name}{s}"),
            Binary::Win(s) => format!("{name}{s}"),
            Binary::WinLib(s) => format!("{name}{s}"),
        }
    }

    pub fn dest_name(&self, name: &str) -> String {
        match self {
            Binary::Lib(s) => format!("lib{name}{s}"),
            Binary::Win(s) => format!("{name}{s}"),
            Binary::WinLib(_) => format!("{name}.lib"),
        }
    }
}

impl OperatingSystem {
    pub const fn name(&self) -> &'static str {
        match self {
            OperatingSystem::Linux => "linux",
            OperatingSystem::Windows => "windows",
            OperatingSystem::Osx => "osx",
        }
    }
    pub const fn shared_artifacts(&self) -> &'static [Binary] {
        match self {
            OperatingSystem::Linux => &[Binary::Lib(".so")],
            OperatingSystem::Windows => &[
                Binary::Win(".pdb"),
                Binary::WinLib(".dll.lib"),
                Binary::Win(".dll"),
                Binary::Win(".dll.exp"),
            ],
            OperatingSystem::Osx => &[Binary::Lib(".dylib")],
        }
    }
    pub const fn static_artifacts(&self) -> &'static [Binary] {
        match self {
            OperatingSystem::Linux => &[Binary::Lib(".a")],
            OperatingSystem::Windows => &[Binary::Win(".lib")],
            OperatingSystem::Osx => &[Binary::Lib(".a")],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Architecture {
    Athena,
    SystemCore,
    X86_64,
    Arm64,
    OsxUniversal,
}

impl Architecture {
    pub const fn name(&self) -> &'static str {
        match self {
            Architecture::Athena => "athena",
            Architecture::SystemCore => "systemcore",
            Architecture::X86_64 => "x86-64",
            Architecture::Arm64 => "arm64",
            Architecture::OsxUniversal => "universal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetInfo {
    /// the rust target triple
    pub triple: &'static str,
    /// the operating system
    pub os: OperatingSystem,
    /// the "architecture" that wpilib maven expects.
    /// This is similar to but not exactly the device's actual cpu arch.
    pub arch: Architecture,
}

impl Target {
    /// information about the target.
    pub const fn info(&self) -> TargetInfo {
        match self {
            Target::LinuxAthena => TargetInfo {
                triple: "arm-unknown-linux-gnueabi",
                os: OperatingSystem::Linux,
                arch: Architecture::Athena,
            },
            Target::LinuxSystemCore => TargetInfo {
                triple: "aarch64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::SystemCore,
            },
            Target::WindowsX86_64 => TargetInfo {
                triple: "x86_64-pc-windows-msvc",
                os: OperatingSystem::Windows,
                arch: Architecture::X86_64,
            },
            Target::WindowsArm64 => TargetInfo {
                triple: "aarch64-pc-windows-msvc",
                os: OperatingSystem::Windows,
                arch: Architecture::Arm64,
            },
            Target::OsxUniversal => TargetInfo {
                triple: "universal-apple-darwin",
                os: OperatingSystem::Osx,
                arch: Architecture::OsxUniversal,
            },
            Target::OsxArm64 => TargetInfo {
                triple: "aarch64-apple-darwin",
                os: OperatingSystem::Osx,
                arch: Architecture::Arm64,
            },
            Target::OsxX86_64 => TargetInfo {
                triple: "x86_64-apple-darwin",
                os: OperatingSystem::Osx,
                arch: Architecture::X86_64,
            },
            Target::LinuxX86_64 => TargetInfo {
                triple: "x86_64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::X86_64,
            },
            Target::LinuxArm64 => TargetInfo {
                triple: "aarch64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::Arm64,
            },
        }
    }

    pub fn build(
        &self,
        build_config: &BuildConfig,
        cargo_flags: &Vec<String>,
    ) -> anyhow::Result<()> {
        let cargo_toml_data = std::fs::read(project_root().join("Cargo.toml"))?;
        let manifest = cargo_toml::Manifest::from_slice(cargo_toml_data.as_slice())?;
        let lib_name = manifest.lib.unwrap().name.unwrap();
        let release = !build_config.is_debug();

        match self {
            Target::LinuxAthena => {
                let roborio_toolchain = locate_roborio_toolchain().expect(&format!(
                    "Could not find roboRIO toolchain, is wpilib {} installed?",
                    year()
                ));
                cargo_build(
                    &self.info().triple,
                    release,
                    &[roborio_toolchain.to_str().unwrap()],
                    cargo_flags,
                )?;
            }
            Target::LinuxSystemCore => {
                let systemcore_toolchain = locate_systemcore_toolchain().expect(&format!(
                    "Could not find SystemCore toolchain, is wpilib {} installed?",
                    SYSTEMCORE_YEAR
                ));
                cargo_build(
                    &self.info().triple,
                    release,
                    &[systemcore_toolchain.to_str().unwrap()],
                    cargo_flags,
                )?;
            }
            Target::OsxUniversal => {
                // osxuniversal needs to build twice and then lipo all the artifacts together
                cargo_build("aarch64-apple-darwin", release, &[], cargo_flags)?;
                cargo_build("x86_64-apple-darwin", release, &[], cargo_flags)?;

                let (debug_release, static_shared) = match build_config {
                    BuildConfig::Shared => ("release", "dylib"),
                    BuildConfig::Static => ("release", "a"),
                    BuildConfig::SharedDebug => ("debug", "dylib"),
                    BuildConfig::StaticDebug => ("debug", "a"),
                };

                std::fs::create_dir_all(
                    target_dir().join(format!("universal-apple-darwin/{debug_release}")),
                )
                .ok();
                lipo(format!("{debug_release}/lib{lib_name}.{static_shared}").as_str())?;
            }
            _other => {
                cargo_build(&self.info().triple, release, &[], cargo_flags)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildConfig {
    Shared,
    Static,
    SharedDebug,
    StaticDebug,
}
impl BuildConfig {
    pub const fn is_static(&self) -> bool {
        match self {
            BuildConfig::Shared => false,
            BuildConfig::Static => true,
            BuildConfig::SharedDebug => false,
            BuildConfig::StaticDebug => true,
        }
    }
    pub const fn is_debug(&self) -> bool {
        match self {
            BuildConfig::Shared => false,
            BuildConfig::Static => false,
            BuildConfig::SharedDebug => true,
            BuildConfig::StaticDebug => true,
        }
    }
    pub const fn suffix(&self) -> &'static str {
        match self {
            BuildConfig::Shared => "",
            BuildConfig::Static => "static",
            BuildConfig::SharedDebug => "debug",
            BuildConfig::StaticDebug => "staticdebug",
        }
    }
}

fn lipo(artifact_path: &str) -> anyhow::Result<()> {
    Command::new("lipo")
        .current_dir(project_root())
        .arg("-create")
        .arg("-output")
        .arg(format!("target/universal-apple-darwin/{artifact_path}"))
        .arg(format!("target/x86_64-apple-darwin/{artifact_path}"))
        .arg(format!("target/aarch64-apple-darwin/{artifact_path}"))
        .status()?;
    Ok(())
    // lipo -create -output target/universal-apple-darwin/debug/librdxusb.a
}

fn append_to_path_variable(path: &str, entry: &str) -> String {
    #[cfg(unix)]
    {
        format!("{entry}:{path}")
    }
    #[cfg(windows)]
    {
        format!("{entry};{path}")
    }
}

fn cargo_build(
    triple: &str,
    release: bool,
    path_env: &[&str],
    flags: &Vec<String>,
) -> anyhow::Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cargo = Command::new(cargo);
    cargo.current_dir(project_root());
    cargo.arg("build");
    if release {
        cargo.arg("--release");
    }
    cargo.arg(format!("--target={triple}"));
    cargo.args(flags);

    let mut path = std::env::var("PATH")?;
    for path_addition in path_env {
        path = append_to_path_variable(path.as_str(), path_addition);
    }
    cargo.env("PATH", path);
    cargo.status()?;

    Ok(())
}

pub fn calc_hashes(file_path: &Path) -> anyhow::Result<()> {
    let data = std::fs::read(file_path)?;
    let ext = file_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    std::fs::write(
        file_path.with_extension(format!("{ext}.md5")),
        format!("{:x}", md5::compute(&data)),
    )?;
    let mut h = sha1::Sha1::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha1")),
        format!("{:x}", h.finalize()),
    )?;
    let mut h = sha2::Sha256::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha256")),
        format!("{:x}", h.finalize()),
    )?;
    let mut h = sha2::Sha512::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha512")),
        format!("{:x}", h.finalize()),
    )?;

    Ok(())
}
#[cfg(unix)]
const PATH_SEP: &str = "/";
#[cfg(windows)]
const PATH_SEP: &str = "\\";

pub fn build_maven(
    target: Target,
    group_id: &str,
    artifact_id: &str,
    build_configs: &[BuildConfig],
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    eprintln!("Building target {target:?} with {build_configs:?}");
    let version = version();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));
    let lib_name = CARGO_TOML_INFO.clone().lib.unwrap().name.unwrap();
    let target_info = target.info();

    let maven = target_dir()
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version);
    eprintln!("Creating maven target {maven:?}");

    std::fs::create_dir_all(&maven).ok();
    for build_config in build_configs {
        target.build(build_config, cargo_flags)?;
        let zipfname = maven.join(format!(
            "{artifact_id}-{version}-{}{}{}.zip",
            target_info.os.name(),
            target_info.arch.name(),
            build_config.suffix(),
        ));
        eprintln!("Building zip {zipfname:?}");

        let zipf = std::fs::File::create(&zipfname)?;
        let mut zip = zip::ZipWriter::new(zipf);

        zip.start_file("LICENSE.txt", zip_options())?;
        zip.write_all(std::fs::read(project_root().join("LICENSE.txt"))?.as_slice())?;

        // create the os/arch/linkage/ directory
        zip.add_directory(target_info.os.name(), zip_options())?;
        zip.add_directory(
            format!("{}/{}", target_info.os.name(), target_info.arch.name()),
            zip_options(),
        )?;
        let shared_or_static = if build_config.is_static() {
            "static"
        } else {
            "shared"
        };
        let base_path = format!(
            "{}/{}/{}",
            target_info.os.name(),
            target_info.arch.name(),
            shared_or_static
        );
        zip.add_directory(&base_path, zip_options())?;

        let artifacts = if build_config.is_static() {
            target_info.os.static_artifacts()
        } else {
            target_info.os.shared_artifacts()
        };
        let build_dir = target_dir()
            .join(target_info.triple)
            .join(if build_config.is_debug() {
                "debug"
            } else {
                "release"
            });
        // write the artifact to the zip
        for artifact_bin in artifacts {
            let artifact_name = artifact_bin.source_name(lib_name.as_str());
            let artifact_dest = artifact_bin.dest_name(lib_name.as_str());

            zip.start_file_from_path(format!("{}/{}", &base_path, &artifact_dest), zip_options())?;
            zip.write_all(std::fs::read(build_dir.join(artifact_name))?.as_slice())?;
        }
        zip.finish()?;
        calc_hashes(&zipfname)?;
    }
    Ok(())
}

pub fn build_maven_zip(
    root_path: &Path,
    group_id: &str,
    artifact_id: &str,
    artifact_name: &str,
) -> anyhow::Result<()> {
    let version = version();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = target_dir()
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version);
    std::fs::create_dir_all(&maven).ok();
    let zipfname = &maven.join(format!("{artifact_id}-{version}-{artifact_name}.zip"));
    let zipf = std::fs::File::create(zipfname)?;
    let mut zip = zip::ZipWriter::new(zipf);
    zip.start_file("LICENSE.txt", zip_options())?;
    zip.write_all(std::fs::read(project_root().join("LICENSE.txt"))?.as_slice())?;

    for entry in walkdir::WalkDir::new(root_path).into_iter() {
        let ent = entry?;
        if ent.path() == root_path {
            continue;
        }
        let Ok(relpath) = ent.path().strip_prefix(root_path) else {
            continue;
        };

        if ent.file_type().is_file() {
            zip.start_file_from_path(relpath, zip_options())?;
            zip.write_all(std::fs::read(ent.path())?.as_slice())?;
        } else if ent.file_type().is_dir() {
            zip.add_directory_from_path(ent.path(), zip_options())?;
        }
    }
    zip.finish()?;
    calc_hashes(&zipfname)?;
    Ok(())
}

pub fn build_maven_metadata(group_id: &str, artifact_id: &str) -> anyhow::Result<()> {
    eprintln!("Building maven-metadata.xml file");
    let version = version();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = target_dir()
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id);
    std::fs::create_dir_all(&maven).ok();

    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let maven_metadata = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<metadata>
  <groupId>{group_id}</groupId>
  <artifactId>{artifact_id}</artifactId>
  <versioning>
    <latest>{version}</latest>
    <release>{version}</release>
    <versions>
      <version>{version}</version>
    </versions>
    <lastUpdated>{ts}</lastUpdated>
  </versioning>
</metadata>"
    );
    let maven_metadata_path = maven.join("maven-metadata.xml");
    std::fs::write(&maven_metadata_path, maven_metadata)?;
    calc_hashes(maven_metadata_path.as_path())?;
    Ok(())
}

pub fn build_maven_pom(group_id: &str, artifact_id: &str) -> anyhow::Result<()> {
    eprintln!("Building POM file");
    let version = version();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", "/")));

    let maven = target_dir()
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version);
    std::fs::create_dir_all(&maven).ok();

    let maven_pom_data = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<project xsi:schemaLocation=\"http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd\" xmlns=\"http://maven.apache.org/POM/4.0.0\"
    xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">
  <modelVersion>4.0.0</modelVersion>
  <groupId>{group_id}</groupId>
  <artifactId>{artifact_id}</artifactId>
  <version>{version}</version>
  <packaging>pom</packaging>
</project>"
    );
    let maven_pom_path = maven.join(format!("{artifact_id}-{version}.pom"));
    std::fs::write(&maven_pom_path, maven_pom_data)?;
    calc_hashes(maven_pom_path.as_path())?;
    Ok(())
}
