"""Rebuild the offline licence archive that is embedded as Windows resource 101."""
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-gnu"


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def main():
    metadata = json.loads(command("cargo", "metadata", "--locked", "--format-version", "1",
                                  "--filter-platform", TARGET))
    tree = command("cargo", "tree", "--locked", "--target", TARGET, "-e", "normal",
                   "--prefix", "none", "--format", "{p}")
    used = {line.split(" v")[0] for line in tree.splitlines()}
    files = {"data/LICENSE": ROOT / "data" / "UPSTREAM_LICENSE"}
    sysroot = Path(command("rustc", "--print", "sysroot"))
    files["rust/COPYRIGHT-library.html"] = sysroot / "share/doc/rust/COPYRIGHT-library.html"
    for pkg in metadata["packages"]:
        if pkg["name"] not in used or not pkg["source"]:
            continue
        crate = Path(pkg["manifest_path"]).parent
        licences = sorted(p for p in crate.iterdir() if p.is_file() and
                          p.name.lower().startswith(("licen", "copy", "notice", "unlicen")))
        if not licences:
            raise RuntimeError(f"Missing licence for {pkg['name']}")
        for path in licences:
            files[f"{pkg['name']}-{pkg['version']}/{path.name}"] = path
    with zipfile.ZipFile(ROOT / "windows/third-party-notices.zip", "w") as archive:
        for name, path in sorted(files.items()):
            archive.writestr(zipfile.ZipInfo(name), path.read_bytes(),
                             compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


if __name__ == "__main__":
    main()
