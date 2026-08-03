use anyhow::anyhow;
use sha1::Digest;
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use zip::write::SimpleFileOptions;

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
}

#[derive(Debug)]
pub struct ReduxFIFOCrate {
    /// workspace root
    pub workspace_root: PathBuf,
    /// target directory
    pub target_dir: PathBuf,
    /// Cargo.toml manifest
    pub manifest: cargo_toml::Manifest,
    /// crate version
    pub version: semver::Version,
    /// derive the year to get the version of the wpilib toolchain to use.
    pub year: u64,
}

impl ReduxFIFOCrate {
    pub fn new(workspace_root: &Path) -> anyhow::Result<Self> {
        let workspace_root = workspace_root.to_path_buf();
        let manifest = cargo_toml::Manifest::from_path(workspace_root.join("Cargo.toml"))?;
        let version = manifest
            .workspace
            .as_ref()
            .ok_or(anyhow!("ReduxFIFO Cargo.toml isn't a workspace"))?
            .package
            .as_ref()
            .ok_or(anyhow!("ReduxFIFO Cargo.toml lacks top-level package"))?
            .version
            .clone()
            .ok_or(anyhow!(
                "ReduxFIFO Cargo.toml is not in 'year-semver' format"
            ))?;
        let year = version.major;
        let target_dir = workspace_root.join("target");

        Ok(Self {
            workspace_root,
            target_dir,
            manifest,
            version,
            year,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolchain {
    /// Base directory containing the tools
    pub base: PathBuf,
    /// String prefix, e.g. `aarch64-systemcoreYEAR-linux-gnu`
    pub prefix: String,
}

impl Toolchain {
    pub fn tool(&self, name: &str) -> PathBuf {
        self.base.join(format!("{}-{}", self.prefix, name))
    }

    pub fn locate(prefix: String, arch: &str, year: u64) -> Result<Self, ToolchainNotFound> {
        let gcc_name = format!("{prefix}-gcc");
        if let Ok(w) = which::which(&gcc_name) {
            // sometimes the systemcore toolchain is already in PATH (e.g. in buildserver containers)
            return Ok(Self {
                base: w.parent().unwrap().to_path_buf(),
                prefix,
            });
        }

        let mut search_path = Vec::new();
        let maybe_home = std::env::home_dir();
        if let Some(home) = &maybe_home {
            // We'll prefer the gradle cache, if it exists.
            search_path.push(home.join(format!(".gradle/toolchains/first/{year}/{arch}/bin")));
        }

        #[cfg(unix)]
        {
            if let Some(home) = &maybe_home {
                // All unicies have their wpilib install in the home directory.
                // For now.
                search_path.push(home.join(format!("wpilib/{year}/{arch}/bin")));
            };
            // the cross-compiler container
            search_path.push(PathBuf::from("/usr/local/bin"));
        }

        #[cfg(windows)]
        {
            // windows typically puts the toolchain in C:\Users\Public for whatever reason
            search_path.push(
                PathBuf::from(std::env::var("PUBLIC").unwrap_or("C:\\Users\\Public".into()))
                    .join(format!("wpilib\\{year}\\{arch}\\bin")),
            );
            if let Some(home) = &maybe_home {
                search_path.push(home.join(format!("wpilib\\{year}\\{arch}\\bin")));
            }
        }

        // find gcc base path
        for path in &search_path {
            let gcc = path.join(&gcc_name);
            if gcc.exists() && gcc.is_file() {
                return Ok(Self {
                    base: path.clone(),
                    prefix,
                });
            }
        }
        Err(ToolchainNotFound {
            prefix,
            searched: search_path,
        })
    }
}

#[derive(Debug)]
pub struct ToolchainNotFound {
    pub prefix: String,
    pub searched: Vec<PathBuf>,
}
impl core::fmt::Display for ToolchainNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Toolchain {} not found, is it installed?", self.prefix)?;
        writeln!(f, "Searched in:")?;
        for path in &self.searched {
            writeln!(f, "\t{}", path.display())?;
        }
        Ok(())
    }
}
impl core::error::Error for ToolchainNotFound {}

pub fn locate_systemcore_toolchain(year: u64) -> Result<Toolchain, ToolchainNotFound> {
    Toolchain::locate(
        format!("aarch64-systemcore{year}-linux-gnu"),
        "systemcore",
        year,
    )
}

pub fn locate_aarch64_toolchain(year: u64) -> Result<Toolchain, ToolchainNotFound> {
    Toolchain::locate(format!("aarch64-trixie-linux-gnu"), "arm64", year)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
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
    SystemCore,
    X86_64,
    Arm64,
    OsxUniversal,
}

impl Architecture {
    pub const fn name(&self) -> &'static str {
        match self {
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
        crate_info: &ReduxFIFOCrate,
        build_config: &BuildConfig,
        cargo_flags: &Vec<String>,
    ) -> anyhow::Result<()> {
        let lib_name = crate_info.manifest.clone().lib.unwrap().name.unwrap();
        let dir = crate_info.workspace_root.as_path();
        let release = !build_config.is_debug();

        match self {
            Target::LinuxSystemCore => {
                cargo_build(
                    dir,
                    &self.info().triple,
                    release,
                    Some(&locate_systemcore_toolchain(crate_info.year)?),
                    cargo_flags,
                )?;
            }
            Target::LinuxArm64 => {
                cargo_build(
                    dir,
                    &self.info().triple,
                    release,
                    Some(&locate_aarch64_toolchain(crate_info.year)?),
                    cargo_flags,
                )?;
            }
            Target::OsxUniversal => {
                // osxuniversal needs to build twice and then lipo all the artifacts together
                cargo_build(dir, "aarch64-apple-darwin", release, None, cargo_flags)?;
                cargo_build(dir, "x86_64-apple-darwin", release, None, cargo_flags)?;

                let (debug_release, static_shared) = match build_config {
                    BuildConfig::Shared => ("release", "dylib"),
                    BuildConfig::Static => ("release", "a"),
                    BuildConfig::SharedDebug => ("debug", "dylib"),
                    BuildConfig::StaticDebug => ("debug", "a"),
                };

                std::fs::create_dir_all(
                    crate_info
                        .target_dir
                        .join(format!("universal-apple-darwin/{debug_release}")),
                )
                .ok();
                lipo(
                    dir,
                    format!("{debug_release}/lib{lib_name}.{static_shared}").as_str(),
                )?;
            }
            _other => {
                cargo_build(dir, &self.info().triple, release, None, cargo_flags)?;
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

fn lipo(dir: &Path, artifact_path: &str) -> anyhow::Result<()> {
    Command::new("lipo")
        .current_dir(dir)
        .arg("-create")
        .arg("-output")
        .arg(format!("target/universal-apple-darwin/{artifact_path}"))
        .arg(format!("target/x86_64-apple-darwin/{artifact_path}"))
        .arg(format!("target/aarch64-apple-darwin/{artifact_path}"))
        .status()?;
    Ok(())
}

fn cargo_build(
    dir: &Path,
    triple: &str,
    release: bool,
    link_toolchain: Option<&Toolchain>,
    flags: &Vec<String>,
) -> anyhow::Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cargo = Command::new(cargo);
    cargo.current_dir(dir);
    cargo.arg("build");
    if release {
        cargo.arg("--release");
    }
    cargo.arg(format!("--target={triple}"));
    cargo.args(flags);

    // prepend the toolchain to PATH and setup environment variables
    if let Some(toolchain) = link_toolchain {
        #[cfg(unix)]
        const SEP: &str = ":";
        #[cfg(windows)]
        const SEP: &str = ";";

        let mut path = std::env::var("PATH")?;
        path = format!("{}{SEP}{path}", toolchain.base.display());

        // convert from kebab to UPPER_SNAKE
        let var_target = triple.replace("-", "_").to_uppercase();
        cargo.env(
            format!("CARGO_TARGET_{var_target}_LINKER"),
            toolchain.tool("gcc"),
        );
        cargo.env(
            format!("CARGO_TARGET_{var_target}_AR"),
            toolchain.tool("ar"),
        );
        cargo.env("PATH", path);
    }
    cargo.status()?;

    Ok(())
}

struct LowerHexAdapter<T>(T);
impl<T: AsRef<[u8]>> core::fmt::Display for LowerHexAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.0.as_ref() {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
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
        LowerHexAdapter(h.finalize()).to_string(),
    )?;
    let mut h = sha2::Sha256::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha256")),
        LowerHexAdapter(h.finalize()).to_string(),
    )?;
    let mut h = sha2::Sha512::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha512")),
        LowerHexAdapter(h.finalize()).to_string(),
    )?;

    Ok(())
}
#[cfg(unix)]
const PATH_SEP: &str = "/";
#[cfg(windows)]
const PATH_SEP: &str = "\\";

pub fn build_maven(
    crate_info: &ReduxFIFOCrate,
    target: Target,
    group_id: &str,
    artifact_id: &str,
    build_configs: &[BuildConfig],
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    eprintln!("Building target {target:?} with {build_configs:?}");
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));
    let lib_name = crate_info.manifest.clone().lib.unwrap().name.unwrap();
    let target_info = target.info();

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&crate_info.version.to_string());
    eprintln!("Creating maven target {maven:?}");

    std::fs::create_dir_all(&maven).ok();
    for build_config in build_configs {
        target.build(crate_info, build_config, cargo_flags)?;
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
        zip.write_all(std::fs::read(crate_info.workspace_root.join("LICENSE.txt"))?.as_slice())?;

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
        let build_dir =
            crate_info
                .target_dir
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
    crate_info: &ReduxFIFOCrate,
    root_path: &Path,
    group_id: &str,
    artifact_id: &str,
    artifact_name: &str,
) -> anyhow::Result<()> {
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version.to_string());
    std::fs::create_dir_all(&maven).ok();
    let zipfname = &maven.join(format!("{artifact_id}-{version}-{artifact_name}.zip"));
    let zipf = std::fs::File::create(zipfname)?;
    let mut zip = zip::ZipWriter::new(zipf);
    zip.start_file("LICENSE.txt", zip_options())?;
    zip.write_all(std::fs::read(crate_info.workspace_root.join("LICENSE.txt"))?.as_slice())?;

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

pub fn build_maven_metadata(
    crate_info: &ReduxFIFOCrate,
    group_id: &str,
    artifact_id: &str,
) -> anyhow::Result<()> {
    eprintln!("Building maven-metadata.xml file");
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = crate_info
        .target_dir
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

pub fn build_maven_pom(
    crate_info: &ReduxFIFOCrate,
    group_id: &str,
    artifact_id: &str,
) -> anyhow::Result<()> {
    eprintln!("Building POM file");
    let version = crate_info.version.clone();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", "/")));

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version.to_string());
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
