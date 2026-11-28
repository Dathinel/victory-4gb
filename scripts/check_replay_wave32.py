"""Check replay/translate parity for one generated Wave64 shader. Pass a diagnostics victory.exe."""
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile
import zlib


def pack(fmt, *values):
    return struct.pack("<" + fmt, *values)


def container():
    # Wave64 compute group (1024, 1, 1), 32 KiB LDS. Elect a lane, reserve
    # through ds_add_rtn_u32, broadcast its ticket, and write the result.
    words = (
        0xbe850380, 0xf4080202, 0xfa000000, 0x34080082, 0x7e0202ff, 64,
        0x7e0402ff, 32764, 0x7e0a0280, 0xd8347000, 0x0504, 0xbf8a0000,
        0x7e0602ff, 0x55555555, 0x360c0000, 0x7d880c01, 0xbe8c246a,
        0xbf880002, 0xd8800000, 0x03000102, 0xbefe040c, 0x7e0e0503,
        0x4a060c07, 0xbf8a0000, 0xd8d80000, 0x05000002, 0xe0701000,
        0x80020304, 0x4a0c08ff, 4096, 0xe0701000, 0x80020506, 0xbf810000,
    )
    code = pack("I" * len(words), *words)
    record = 0x68
    stage = record + 0x54
    start = stage + 0x34
    tables = start + len(code)
    data = bytearray(tables)
    struct.pack_into("<Q", data, 0, 6)
    for offset in (8, 16, 24, 32, 40, 48, 72):
        struct.pack_into("<Q", data, offset, 0xdeadbeefdeadbeef)
    struct.pack_into("<Q", data, 96, record)
    struct.pack_into("<QQ", data, record, start, len(code))
    struct.pack_into("<QQII", data, record + 40, tables, tables + 40, 2, 1)
    struct.pack_into("<I", data, record + 80, 16)
    struct.pack_into("<IIII", data, stage + 36, 10 | (64 << 15), 1024, 1, 1)
    data[start:] = code
    data += pack("IIIIBBxx", 0, 0, 4, 0, 0, 0)
    data += pack("IIIIBBxx", 3, 0, 1, 0, 4, 1)
    data += pack("BBxxIIIBBBx", 3, 3, 0, 0, 0, 4, 4, 1)
    raw = b"FSSM" + pack("IQ", 0x101, len(data)) + data
    compressed = zlib.compress(raw)
    chunk = b"XBCH" + pack("IQQ", 3, len(compressed), len(raw)) + compressed
    part = bytes(24) + b"XDAT" + pack("IIIII", 1, 36, 13, 48, len(chunk)) + chunk
    return b"DXBC" + bytes(16) + pack("IIII", 1, len(part) + 44, 1, 36) + b"DXIL" + pack("I", len(part)) + part


def capture_frame(key):
    # replay/wire.rs v1. Only title_shaders reads this frame: it needs a
    # pipeline name and a compute Code part, with no executable GPU commands.
    name = ("compute " + key).encode("ascii")
    header = b"VICFRAME" + pack("I", 1) + pack("8I", *([0] * 8))
    pipeline = pack("IIIBiII", 1, 0, 1, 1, 6, 0, len(name)) + name
    return header + pipeline + bytes(7 * 4)


def check(exe, root):
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith("VICTORY_")}
    env.update(VICTORY_HOST_MAX_WAVE="32", VICTORY_HOST_RDNA="0")
    source = root / "synthetic.dxbc"
    source.write_bytes(container())
    command = [exe, "translate", "--list", "--jobs", "1", source]
    listing = subprocess.check_output(command, env=env, text=True)
    key = re.search(r"#([0-9a-f]{16}) .* Cs  emitted ", listing)
    if not key:
        raise AssertionError(listing)
    key = key.group(1)
    frame = root / "capture-1"
    frame.mkdir()
    (frame / "frame.bin").write_bytes(capture_frame(key))
    kept = root / "containers"
    kept.mkdir()
    (kept / (key + ".cdxil")).write_bytes(source.read_bytes())
    results = []
    for mode in ("normal", "paired"):
        if mode == "paired":
            env["VICTORY_WAVE_PAIRS_ONLY"] = key
        dump = root / ("translate-" + mode)
        dump.mkdir()
        subprocess.run(command, env=dict(env, VICTORY_EMIT_DUMP=str(dump)), check=True)
        (output,) = dump.glob("*.dxil")
        replay = root / ("replay-" + mode)
        subprocess.run([exe, "replay", frame, "--retranslate", replay], env=env, check=True)
        actual = (replay / (key + ".dxil")).read_bytes()
        if actual != output.read_bytes():
            raise AssertionError(f"{mode}: replay and translate emitted different DXIL")
        results.append(actual)
    if results[0] == results[1]:
        raise AssertionError("selecting the synthetic shader did not change its translation")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as directory:
        check(Path(sys.argv[1]).resolve(strict=True), Path(directory))
    print("Normal and paired translations matched replay byte for byte.")
