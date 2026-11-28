//! CLI translation parity on generated Wave64 compute programs.
use std::{fs, path::Path, process::Command};
use tempfile::tempdir;

#[cfg(all(windows, feature = "diagnostics"))]
const MESH: &[u32] = &[
    0xbe850380, 0xf4080202, 0xfa000000, 0x34080082, 0x7e0202ff, 64, 0x7e0402ff, 32764, 0x7e0a0280,
    0xd8347000, 0x0504, 0xbf8a0000, 0x7e0602ff, 0x55555555, 0x360c0000, 0x7d880c01, 0xbe8c246a,
    0xbf880002, 0xd8800000, 0x03000102, 0xbefe040c, 0x7e0e0503, 0x4a060c07, 0xbf8a0000, 0xd8d80000,
    0x05000002, 0xe0701000, 0x80020304, 0x4a0c08ff, 4096, 0xe0701000, 0x80020506, 0xbf810000,
];

const REDUCE: &[u32] = &[
    0xbe850380, 0xf4080202, 0xfa000000, 0x34080082, 0x7e0a0281, 0x2c0c0081, 0x340e0b06, 0x7d880000,
    0xbe8c246a, 0xbf840017, 0xbe8425c1, 0xd5010001, 0x00120e80, 0x380202e9, 0xde54c101, 0x380202e9,
    0x93601201, 0x380202e9, 0x00092401, 0x380202fa, 0xff094001, 0xd7780002, 0x02010101, 0x38020302,
    0xd7600007, 0x00014101, 0x38020207, 0xd7600007, 0x00010101, 0xbefe0404, 0x7e000207, 0xe0701000,
    0x80020004, 0xbefe040c, 0xbf810000,
];

fn words(out: &mut Vec<u8>, values: &[u32]) {
    out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
}

fn compute(code: &[u32], threads: u32, lds_blocks: u32) -> Vec<u8> {
    let record = 0x68;
    let stage = record + 0x54;
    let start = stage + 0x34;
    let tables = start + 4 * code.len();
    let mut data = vec![0; start];
    for (off, v) in [
        (0, 6u64),
        (96, record as u64),
        (record, start as u64),
        (record + 8, (code.len() * 4) as u64),
        (record + 40, tables as u64),
        (record + 48, (tables + 40) as u64),
    ] {
        data[off..off + 8].copy_from_slice(&v.to_le_bytes());
    }
    for off in [8, 16, 24, 32, 40, 48, 72] {
        data[off..off + 8].copy_from_slice(&0xdeadbeefdeadbeefu64.to_le_bytes());
    }
    for (off, v) in [
        (record + 56, 2u32),
        (record + 60, 1),
        (record + 80, 16),
        (stage + 36, 10 | (lds_blocks << 15)),
        (stage + 40, threads),
        (stage + 44, 1),
        (stage + 48, 1),
    ] {
        data[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }
    words(&mut data, code);
    // Two buffer descriptors and their user-data layout.
    words(
        &mut data,
        &[0, 0, 4, 0, 0, 3, 0, 1, 0, 0x104, 0x303, 0, 0, 0, 0x10404],
    );
    let mut raw = b"FSSM".to_vec();
    words(&mut raw, &[0x101, data.len() as u32, 0]);
    raw.extend(data);
    let mut storage = vec![0; zlib_rs::compress_bound(raw.len())];
    let (packed, status) =
        zlib_rs::compress_slice(&mut storage, &raw, zlib_rs::DeflateConfig::new(6));
    assert_eq!(status, zlib_rs::ReturnCode::Ok);
    let mut chunk = b"XBCH".to_vec();
    words(
        &mut chunk,
        &[3, packed.len() as u32, 0, raw.len() as u32, 0],
    );
    chunk.extend_from_slice(packed);
    let mut part = vec![0; 24];
    part.extend(b"XDAT");
    words(&mut part, &[1, 36, 13, 48, chunk.len() as u32]);
    part.extend(chunk);
    let mut container = b"DXBC".to_vec();
    container.resize(20, 0);
    words(&mut container, &[1, part.len() as u32 + 44, 1, 36]);
    container.extend(b"DXIL");
    words(&mut container, &[part.len() as u32]);
    container.extend(part);
    container
}

fn cli(root: &Path, args: &[&str], selected: Option<&str>, dump: &Path) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_victory"));
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("VICTORY_")
        {
            cmd.env_remove(key);
        }
    }
    if let Some(key) = selected {
        cmd.env("VICTORY_WAVE_PAIRS_ONLY", key);
    }
    let output = cmd
        .current_dir(root)
        .args(args)
        .env("VICTORY_HOST_MAX_WAVE", "32")
        .env("VICTORY_HOST_RDNA", "0")
        .env("VICTORY_CACHE_DIR", root.join("cache"))
        .env("VICTORY_DATA_DIR", root.join("data"))
        .env("VICTORY_EMIT_DUMP", dump)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn translate(root: &Path, inputs: &[&str], selected: Option<&str>) -> (Vec<Vec<u8>>, String) {
    let dump = tempfile::tempdir_in(root).unwrap();
    let mut args = vec!["translate", "--list", "--jobs", "1"];
    args.extend_from_slice(inputs);
    let report = cli(root, &args, selected, dump.path());
    let mut files = Vec::new();
    for entry in fs::read_dir(dump.path()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "dxil") {
            files.push(fs::read(path).unwrap());
        }
    }
    files.sort();
    (files, report)
}

#[test]
fn batch_preserves_distinct_header_policies_and_skips_identical_copies() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let mut bytes = compute(REDUCE, 64, 0);
    for name in ["a.dxbc", "copy.dxbc"] {
        fs::write(root.join(name), &bytes).unwrap();
    }
    bytes[4..20].copy_from_slice(&[
        0x58, 0x1d, 0xab, 0x2c, 0xd8, 0xbc, 0xf2, 0x2e, 0x5c, 0x96, 0x4b, 0x98, 0xe9, 0x03, 0x48,
        0xd1,
    ]);
    fs::write(root.join("scene.dxbc"), bytes).unwrap();
    let (first, _) = translate(root, &["a.dxbc"], None);
    let (second, _) = translate(root, &["scene.dxbc"], None);
    assert_eq!((first.len(), second.len()), (1, 1));
    assert_ne!(first, second);
    let mut expected = [first.clone(), second].concat();
    expected.sort();
    for inputs in [["a.dxbc", "scene.dxbc"], ["scene.dxbc", "a.dxbc"]] {
        assert_eq!(translate(root, &inputs, None).0, expected);
    }
    let (copies, report) = translate(root, &["a.dxbc", "copy.dxbc"], None);
    assert_eq!(copies, first);
    assert!(report.contains("1 duplicates skipped"), "{report}");
}

#[cfg(all(windows, feature = "diagnostics"))]
#[test]
fn replay_and_translate_select_the_same_wave32_form() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let bytes = compute(MESH, 1024, 64);
    let key = blake3::hash(&bytes).to_hex()[..16].to_owned();
    fs::write(root.join("s.dxbc"), &bytes).unwrap();
    fs::create_dir(root.join("containers")).unwrap();
    fs::write(root.join("containers").join(format!("{key}.cdxil")), bytes).unwrap();
    let name = format!("compute {key}");
    let mut frame = b"VICFRAME".to_vec();
    words(&mut frame, &[1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1]);
    frame.push(1);
    words(&mut frame, &[6, 0, name.len() as u32]);
    frame.extend(name.bytes());
    frame.extend([0; 28]);
    fs::create_dir(root.join("capture-1")).unwrap();
    fs::write(root.join("capture-1/frame.bin"), frame).unwrap();
    let mut outputs = Vec::new();
    for (directory, selected) in [("normal", None), ("paired", Some(key.as_str()))] {
        let (translation, _) = translate(root, &["s.dxbc"], selected);
        assert_eq!(translation.len(), 1);
        cli(
            root,
            &["replay", "capture-1", "--retranslate", directory],
            selected,
            root,
        );
        let replay = fs::read(root.join(directory).join(format!("{key}.dxil"))).unwrap();
        assert_eq!(translation[0], replay);
        outputs.push(replay);
    }
    assert_ne!(outputs[0], outputs[1]);
}
