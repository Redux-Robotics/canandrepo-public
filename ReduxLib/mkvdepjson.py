import json
import os
import sys
import pathlib
import tomllib

MAVEN_URL = "https://maven.reduxrobotics.com/"
JSON_URL = "https://frcsdk.reduxrobotics.com/"
VDEP_ROOT = pathlib.Path(__file__).parent
with open(VDEP_ROOT/"version.txt") as f:
    version = f.read().strip()

with open(VDEP_ROOT/"../reduxfifo/Cargo.toml", "rb") as f:
    reduxfifo_cargo_toml = tomllib.load(f)
    reduxfifo_version = reduxfifo_cargo_toml['workspace']['package']['version']
    print(reduxfifo_version)

versionYear = version.split(".")[0]
fileName = "ReduxLib_" + versionYear + ".json"


if __name__ == "__main__":

    # This flag determines if the Maven repository should be the vendordep's build directory,
    # rather than the production maven URL.
    # 
    # (The build directory gets written on ./gradlew publish, so this ties dependent robot projects on the freshest build.)
    # 
    local_flag = False
    static_flag = False
    if len(sys.argv) > 1:
        if 'local' in sys.argv[1:]:
            local_flag = True
        if 'static' in sys.argv[1:]:
            static_flag = True
    template = {
        "fileName": f"ReduxLib-{version}.json",
        "name": "ReduxLib",
        "version": version,
        "frcYear": versionYear,
        "uuid": "151ecca8-670b-4026-8160-cdd2679ef2bd",
        "mavenUrls": [ MAVEN_URL ] if not local_flag else [
            "file:" + str(VDEP_ROOT/"build/repos/releases"),
            "file:" + str(VDEP_ROOT/"../reduxfifo/target/maven")
        ],
        "jsonUrl": JSON_URL + fileName,
        "javaDependencies": [
            {
                "groupId": "com.reduxrobotics.frc",
                "artifactId": "ReduxLib-java",
                "version": version,  
            }
        ],
        "jniDependencies": [
            {
                "groupId": "com.reduxrobotics.frc",
                "artifactId": "ReduxLib-fifo",
                "version": reduxfifo_version,
                "isJar": False,
                "skipInvalidPlatforms": True,
                "validPlatforms": [
                    "linuxathena",
                    "linuxx86-64",
                    "linuxarm64",
                    "osxuniversal",
                    "windowsx86-64"
                ]
            }
        ],
        "cppDependencies": [
            {
                "groupId": "com.reduxrobotics.frc",
                "artifactId": "ReduxLib-cpp",
                "version": version,
                "libName": "ReduxLib",
                "headerClassifier": "headers",
                "sourcesClassifier": "sources",
                "sharedLibrary": not static_flag,
                "skipInvalidPlatforms": True,
                "binaryPlatforms": [
                    "linuxathena",
                    "linuxx86-64",
                    "linuxarm64",
                    "osxuniversal",
                    "windowsx86-64"
                ]
            },
            {
                "groupId": "com.reduxrobotics.frc",
                "artifactId": "ReduxLib-fifo",
                "version": reduxfifo_version,
                "libName": "reduxfifo",
                "headerClassifier": "headers",
                "sharedLibrary": not static_flag,
                "skipInvalidPlatforms": True,
                "binaryPlatforms": [
                    "linuxathena",
                    "linuxx86-64",
                    "linuxarm64",
                    "osxuniversal",
                    "windowsx86-64"
                ]
            }
        ]
    }
    print(json.dumps(template, indent=2))

    if not local_flag:
        with open(fileName, "w") as f:
            json.dump(template, f, indent=2)
        with open("build/allOutputs/" + fileName, "w") as f:
            json.dump(template, f, indent=2)