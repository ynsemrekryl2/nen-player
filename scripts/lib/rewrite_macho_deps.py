#!/usr/bin/env python3
"""NEN-043: geçişli Homebrew dylib kapanışını bulur, Frameworks dizinine
kopyalar ve tüm Homebrew yüklerini @rpath / @loader_path'e çevirir.

    rewrite_macho_deps.py <binary> <frameworks_dir>

Vendor önekleri (varsayılan `/opt/homebrew:/usr/local`)
`NEN_BUNDLE_VENDOR_PREFIXES` ile değiştirilebilir (`:`-ayrılmış) — yalnız
`scripts/tests/bundle-macos.test.sh` gerçek Homebrew'a dokunmadan sentetik
bir zincirle sınamak için kullanır.

Her gömülü dylib için stdout'a bir satır basar:

    <kaynak realpath>\t<bundle içindeki basename>

`bash`'in kendisi bash 3.2 (macOS'un sistem kabuğu) olduğu ve ilişkisel
dizi desteklemediği için graf işi burada — yeniden çalıştırılabilir, tek
komutla test edilebilir bir yardımcı script'te.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys

VENDOR_PREFIXES = [
    p.rstrip("/") + "/"
    for p in os.environ.get("NEN_BUNDLE_VENDOR_PREFIXES", "/opt/homebrew:/usr/local").split(":")
    if p
]


def is_vendor_path(path: str) -> bool:
    return any(path.startswith(pref) for pref in VENDOR_PREFIXES)


def otool_deps(path: str) -> list[str]:
    out = subprocess.run(["otool", "-L", path], capture_output=True, text=True, check=True).stdout
    deps = []
    for line in out.splitlines()[1:]:
        line = line.strip()
        if not line:
            continue
        deps.append(line.split(" (compatibility", 1)[0])
    return deps


def dylib_id(path: str) -> str:
    out = subprocess.run(["otool", "-D", path], capture_output=True, text=True, check=True).stdout
    lines = [l.strip() for l in out.splitlines() if l.strip()]
    if len(lines) < 2:
        raise RuntimeError(f"{path}: LC_ID_DYLIB yok (otool -D boş döndü)")
    return lines[1]


def run(*args: str) -> None:
    subprocess.run(args, check=True, capture_output=True)


def discover(binary: str) -> dict[str, str]:
    """realpath -> bundle basename, geçişli vendor kapanışı."""
    seen: dict[str, str] = {}
    queue = [binary]
    scanned: set[str] = set()

    while queue:
        cur = queue.pop(0)
        if cur in scanned:
            continue
        scanned.add(cur)
        for dep in otool_deps(cur):
            if not is_vendor_path(dep):
                continue
            real = os.path.realpath(dep)
            if real in seen:
                continue
            base = os.path.basename(dylib_id(real))
            seen[real] = base
            queue.append(real)

    return seen


def copy_into(seen: dict[str, str], frameworks: str) -> None:
    os.makedirs(frameworks, exist_ok=True)
    for real, base in seen.items():
        dst = os.path.join(frameworks, base)
        if os.path.exists(dst):
            if os.path.getsize(dst) != os.path.getsize(real):
                sys.exit(f"HATA: iki farklı dylib aynı basename'e çöktü: {base}")
            continue
        shutil.copy2(real, dst)
        os.chmod(dst, 0o755)


def rewrite(target: str, seen: dict[str, str], *, is_copied_dylib: bool) -> None:
    if is_copied_dylib:
        run("install_name_tool", "-id", f"@rpath/{os.path.basename(target)}", target)
    for dep in otool_deps(target):
        if not is_vendor_path(dep):
            continue
        real = os.path.realpath(dep)
        base = seen.get(real)
        if base is None:
            sys.exit(f"HATA: {target} şuna bağlı ama kapanışta yok: {dep}")
        run("install_name_tool", "-change", dep, f"@rpath/{base}", target)


def add_rpath(target: str, rpath: str) -> None:
    try:
        run("install_name_tool", "-add_rpath", rpath, target)
    except subprocess.CalledProcessError as exc:
        stderr = (exc.stderr or b"").decode("utf-8", "replace")
        if "would duplicate path" in stderr:
            return
        raise


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(f"kullanım: {sys.argv[0]} <binary> <frameworks_dir>")
    binary, frameworks = sys.argv[1], sys.argv[2]

    seen = discover(binary)
    copy_into(seen, frameworks)

    for real, base in seen.items():
        dst = os.path.join(frameworks, base)
        rewrite(dst, seen, is_copied_dylib=True)
        add_rpath(dst, "@loader_path")

    rewrite(binary, seen, is_copied_dylib=False)
    add_rpath(binary, "@executable_path/../Frameworks")

    for real, base in sorted(seen.items()):
        print(f"{real}\t{base}")


if __name__ == "__main__":
    main()
