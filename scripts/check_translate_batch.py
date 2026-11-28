"""Verify batch translation against individual DXIL outputs using a synthetic Wave64 reduction."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from check_replay_wave32 import compute_container


def check(exe, root):
    # Mask lanes, reduce their values with a wave OR, and store the result.
    words = (
        0xbe850380, 0xf4080202, 0xfa000000, 0x34080082, 0x7e0a0281,
        0x2c0c0081, 0x340e0b06, 0x7d880000, 0xbe8c246a, 0xbf840017,
        0xbe8425c1, 0xd5010001, 0x00120e80, 0x380202e9, 0xde54c101,
        0x380202e9, 0x93601201, 0x380202e9, 0x00092401, 0x380202fa,
        0xff094001, 0xd7780002, 0x02010101, 0x38020302, 0xd7600007,
        0x00014101, 0x38020207, 0xd7600007, 0x00010101, 0xbefe0404,
        0x7e000207, 0xe0701000, 0x80020004, 0xbefe040c, 0xbf810000,
    )
    payload = compute_container(words, threads=64, lds_blocks=0)
    ordinary = root / "ordinary.dxbc"
    scene = root / "scene.dxbc"
    copy = root / "copy.dxbc"
    ordinary.write_bytes(payload)
    copy.write_bytes(payload)
    # Header digest recognized by the scene-kernel translation policy.
    scene.write_bytes(payload[:4] + bytes.fromhex("581dab2cd8bcf22e5c964b98e90348d1") + payload[20:])
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith("VICTORY_")}
    env.update(VICTORY_HOST_MAX_WAVE="32", VICTORY_HOST_RDNA="0",
               VICTORY_CACHE_DIR=str(root / "cache"), VICTORY_DATA_DIR=str(root / "data"))

    def translate(*inputs):
        with tempfile.TemporaryDirectory(dir=root) as dump:
            report = subprocess.check_output(
                [exe, "translate", "--list", "--jobs", "1", *inputs], cwd=root,
                env=dict(env, VICTORY_EMIT_DUMP=dump), text=True,
            )
            outputs = sorted(p.read_bytes() for p in Path(dump).glob("*.dxil"))
            return outputs, report

    first, = translate(ordinary)[0]
    second, = translate(scene)[0]
    if first == second:
        raise AssertionError("individual translations did not exercise two forms")
    for inputs in [(ordinary, scene), (scene, ordinary)]:
        if translate(*inputs)[0] != sorted([first, second]):
            raise AssertionError(f"batch discarded a distinct translation: {inputs}")
    outputs, report = translate(ordinary, copy)
    if outputs != [first] or "1 duplicates skipped" not in report:
        raise AssertionError("identical containers should be translated once")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as directory:
        check(Path(sys.argv[1]).resolve(strict=True), Path(directory))
    print("Both batch orders matched individual translations; identical copies were skipped.")
