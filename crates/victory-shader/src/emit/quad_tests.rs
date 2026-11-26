//! Pixel quad tests adapted from Marvisak/VIctory PR 2817 by 0xkn0wh3r3.
//! The edge readback exercises DPP8 directly on hardware and WARP.

use super::bitcode::{Attr, Term};
use super::container::{self, SigElement};
use super::host::{Draw, Host};
use super::shader::Shader;
use super::{HostShader, emit, validator};
use crate::dxbc::{Container, FourCC};
use crate::native::{HwStage, NativeProgram, PsInput, StageIo, Usage};
use crate::{ShaderError, io, ir, resource, structure};
use windows::Win32::Graphics::Direct3D12::D3D12_SHADER_VISIBILITY_PIXEL;

/// `SPI_PS_INPUT_ADDR` bits: PERSP_CENTER, POS_X_FLOAT and POS_Y_FLOAT.
const PERSP_CENTER: u32 = 1 << 1;
const POS_XY: u32 = 1 << 8 | 1 << 9;

const S_ENDPGM: u32 = 0xBF81_0000;

fn element(name: &str, kind: u8, row: u32) -> SigElement {
    SigElement {
        name: name.into(),
        index: 0,
        kind,
        comp: 9,
        interp: 0,
        row: Some(row),
        col: 0,
        cols: 4,
        usage: 0,
        half: false,
    }
}

/// Translates a pixel program with the input VGPRs `input_addr` enables,
/// the input signature `inputs` and, if `texcoord`, one float4 attribute
/// (attr 0, row 0); it writes target 0.
fn pixel(
    words: &[u32],
    input_addr: u32,
    inputs: &[SigElement],
    texcoord: bool,
) -> Result<HostShader, ShaderError> {
    let attribute = PsInput {
        usage: Usage::Generic,
        row: 0,
        mask: 0b1111,
        attr: 0,
        flat: false,
        fp16: false,
        pack2: false,
        row2: 0,
    };
    let native = NativeProgram {
        stage: HwStage::Ps,
        wave64: false,
        vgprs: 8,
        layout: vec![],
        used: vec![],
        io: StageIo::Ps {
            inputs: if texcoord { vec![attribute] } else { vec![] },
            input_addr,
            db_shader_control: 0,
            patches: vec![],
        },
        code: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
    };
    let mut target = element("SV_Target", 16, 0);
    target.usage = 15;
    let isg1 = container::signature(inputs, true);
    let osg1 = container::signature(&[target], false);
    let bytes = crate::dxbc::build(&[(FourCC::ISG1, &isg1), (FourCC::OSG1, &osg1)]);
    let c = Container::parse(&bytes)?;
    let p = ir::build(&native.patched_code(), native.wave64, native.vgprs)?;
    let st = structure::structure(&p)?;
    let res = resource::track(&p, &native.layout, native.stage, Some(&st))?;
    let io = io::raise(&p, &native)?;
    emit(&c, &native, &p, &st, &io, &res)
}

fn validate(shader: &HostShader) {
    if let Ok(mut v) = validator::Validator::load(validator::Validator::default_path().as_deref()) {
        v.validate(&mut shader.dxil.clone()).unwrap();
    }
}

/// `ds_swizzle_b32 v<dst>, v<src> offset:quad_perm(pattern)`.
fn swizzle(dst: u32, src: u32, pattern: u32) -> [u32; 2] {
    [0xD800_0000 | 0x35 << 18 | 0x8000 | pattern, dst << 24 | src]
}

/// `v_mov_b32_dpp v<dst>, v<src> quad_perm(pattern)`, all rows and banks.
fn mov_quad_perm(dst: u32, src: u32, pattern: u32) -> [u32; 2] {
    [0x7E00_02FA | dst << 17, 0xFF00_0000 | pattern << 8 | src]
}

/// `v_mov_b32_dpp8 v<dst>, v<src>` with lane selections `sel` (three bits
/// per lane of each group of eight).
fn mov_dpp8(dst: u32, src: u32, sel: u32) -> [u32; 2] {
    [0x7E00_02E9 | dst << 17, sel << 8 | src]
}

/// Every quad permutation through `ds_swizzle_b32` and DPP, and DPP8 that
/// keeps each lane in its quad (the same relative selection in both quads), translate and validate; DPP8 that reads across
/// quads has no quad operation and stays refused in a pixel program.
#[test]
fn pixel_quad_swizzles_validate_and_dpp8_cannot_cross_quads() {
    let read = |instruction: [u32; 2]| {
        let words = [
            0xC808_0000, // v_interp_p1_f32 v2, v0, attr0.x
            0xC809_0001, // v_interp_p2_f32 v2, v1, attr0.x
            instruction[0],
            instruction[1],
            0xF800_180F,
            0x0303_0303, // exp mrt0 v3, v3, v3, v3 done vm
            S_ENDPGM,
        ];
        pixel(&words, PERSP_CENTER, &[element("TEXCOORD", 0, 0)], true)
    };
    for pattern in [0x00, 0x55, 0xAA, 0xFF, 0xB1, 0x4E, 0x1B, 0xE4, 0x27] {
        validate(&read(swizzle(3, 2, pattern)).unwrap());
        validate(&read(mov_quad_perm(3, 2, pattern)).unwrap());
    }
    let selections = [3u32, 0, 2, 1, 7, 4, 6, 5];
    let sel = (0..8).fold(0, |s, i| s | selections[i] << (3 * i));
    validate(&read(mov_dpp8(3, 2, sel)).unwrap());
    let mut fetch_inactive = mov_dpp8(3, 2, sel);
    fetch_inactive[0] += 1; // src0 = 0xea, FI enabled
    assert!(read(fetch_inactive).is_err());
    // Change the upper quad alone while keeping its sources local.
    assert!(read(mov_dpp8(3, 2, sel ^ (1 << 12))).is_err());
    // Lane 0 now reads lane 7, in the other quad.
    assert!(read(mov_dpp8(3, 2, sel ^ 4)).is_err());
}

#[test]
fn pixel_dpp8_with_a_predicated_source_is_rejected() {
    let swap_x = [1u32, 0, 3, 2, 5, 4, 7, 6];
    let sel = swap_x.into_iter().enumerate().fold(0, |s, (i, v)| s | v << (3 * i));
    let [first, second] = mov_dpp8(3, 2, sel);
    let words = [
        0xC808_0000, 0xC809_0001, // interpolate v2
        0x7E06_0280, // v_mov_b32 v3, 0
        0x7D88_0481, // v_cmp_gt_u32 vcc, 1, v2
        0xBE80_3C6A, // s_and_saveexec_b32 s0, vcc_lo
        first, second,
        0xBEFE_0300, // s_mov_b32 exec_lo, s0
        0xF800_180F, 0x0303_0303, S_ENDPGM,
    ];
    let result = pixel(&words, PERSP_CENTER, &[element("TEXCOORD", 0, 0)], true);
    eprintln!("masked DPP8: {:?}", result.as_ref().err());
    assert!(result.is_err_and(|e| e.to_string().contains("predicated pixel DPP8")));
}

/// A vertex shader passing a float4 POSITION through to `SV_Position`.
fn pass_position() -> Vec<u8> {
    let mut vs = Shader::new(false);
    let mut input = element("POSITION", 0, 0);
    input.usage = 15;
    vs.inputs.push(input);
    let mut output = element("SV_Position", 3, 0);
    output.usage = 15;
    output.interp = 4;
    vs.outputs.push(output);
    let zero = vs.m.i32(0);
    let undef = vs.m.undef(vs.t.i32);
    for col in 0..4 {
        let col = vs.m.i8(col);
        let args = [zero, zero, col, undef];
        let value = vs.op(
            vs.entry,
            "loadInput.f32",
            Attr::ReadNone,
            vs.t.f32,
            4,
            &args,
        );
        let args = [zero, zero, col, value];
        vs.op(
            vs.entry,
            "storeOutput.f32",
            Attr::NoUnwind,
            vs.t.void,
            5,
            &args,
        );
    }
    vs.m.terminate(vs.entry, Term::Ret);
    vs.finish()
}

/// In whole-quad mode, v2..v5 take the pixel coordinate `reads[k].1` (v0
/// x, v1 y) through quad pattern `reads[k].0`, by `ds_swizzle_b32` or DPP8;
/// target 0 gets v2..v5.
fn quad_reads(dpp: bool, reads: [(u32, u32); 4]) -> HostShader {
    let mut words = vec![0xBEFE_097E]; // s_wqm_b32 exec_lo, exec_lo
    for (dst, (pattern, src)) in (2..).zip(reads) {
        words.extend(if dpp {
            let selectors = (0..8).fold(0, |bits, lane| {
                let source = (lane & 4) | ((pattern >> (2 * (lane & 3))) & 3);
                bits | (source << (3 * lane))
            });
            mov_dpp8(dst, src, selectors)
        } else {
            swizzle(dst, src, pattern)
        });
    }
    words.extend([0xF800_180F, 0x0504_0302, S_ENDPGM]); // exp mrt0 v2..v5
    let shader = pixel(&words, POS_XY, &[element("SV_Position", 3, 0)], false).unwrap();
    validate(&shader);
    shader
}

/// Two triangles sharing the diagonal of a 16 x 16 target, each pixel lit
/// by one: in the quads the diagonal crosses, some lanes are helpers of
/// the triangle drawn. Broadcasts (one lane's value to its quad) and
/// exchanges (across x, across y, diagonal) read the helpers' coordinates
/// as the console's whole-quad mode does, on the hardware adapter and WARP.
#[test]
fn pixel_quad_reads_take_helper_coordinates_at_a_shared_triangle_edge() {
    let _one = super::valu_tests::ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    const SIZE: usize = 16;
    // Pixel centres: lane 0 of a quad is its top left, lane 3 bottom right.
    // A name, the four reads (pattern, coordinate) and the expected texel.
    type Case = (&'static str, [(u32, u32); 4], fn(f32, f32) -> [f32; 4]);
    let cases: [Case; 3] = [
        (
            "broadcast",
            // x of lanes 0 and 1, y of lanes 2 and 3.
            [(0x00, 0), (0x55, 0), (0xAA, 1), (0xFF, 1)],
            |x, y| {
                let (left, bottom) = (x - x % 2.0 + 0.5, y - y % 2.0 + 1.5);
                [left, left + 1.0, bottom, bottom]
            },
        ),
        (
            "exchange",
            // x across x, y across y, x and y across the diagonal.
            [(0xB1, 0), (0x4E, 1), (0x1B, 0), (0x1B, 1)],
            |x, y| {
                let (x, y) = (((x as u32) ^ 1) as f32 + 0.5, ((y as u32) ^ 1) as f32 + 0.5);
                [x, y, x, y]
            },
        ),
        (
            "cycle",
            // Sources [1, 2, 0, 3] rotate three lanes; the inverse differs.
            [(0xc9, 0), (0xc9, 1), (0xc9, 0), (0xc9, 1)],
            |x, y| {
                let lane = (x as usize % 2) + 2 * (y as usize % 2);
                let src = [1usize, 2, 0, 3][lane];
                let sx = x - x % 2.0 + (src % 2) as f32 + 0.5;
                let sy = y - y % 2.0 + (src / 2) as f32 + 0.5;
                [sx, sy, sx, sy]
            },
        ),
    ];
    let vs = pass_position();
    let corners = [
        [-1.0, -1.0, 0.5, 1.0],
        [-1.0, 1.0, 0.5, 1.0],
        [1.0, -1.0, 0.5, 1.0],
        [1.0, 1.0, 0.5, 1.0],
    ];
    let halves = [[0, 1, 2], [1, 2, 3]].map(|t| t.map(|i| corners[i]));
    let mut failures = Vec::new();
    for (name, warp) in [("hardware", false), ("WARP", true)] {
        if !warp && std::env::var("VICTORY_WARP").as_deref() == Ok("1") {
            continue;
        }
        let Ok(host) = Host::new(warp) else {
            eprintln!("no SM 6.6 {name} device; skipped");
            continue;
        };
        for (case, reads, expect) in cases {
            for (instruction, dpp) in [("ds_swizzle", false), ("DPP8", true)] {
                let ps = quad_reads(dpp, reads);
                let root = host
                    .root_signature_for(&[(&ps, D3D12_SHADER_VISIBILITY_PIXEL)])
                    .unwrap();
                let pso = host.pipeline_with(&root, &vs, &ps.dxil).unwrap();
                // One triangle per draw into a cleared target: summed, each
                // pixel has the value of the one triangle that lit it.
                let drawn = halves.map(|triangle| {
                    let draw = Draw {
                        roots: Vec::new(),
                        textures: &[],
                        samplers: &[],
                        vertices: triangle.as_flattened(),
                        stride: 16,
                        size: (SIZE as u32, SIZE as u32),
                        mesh_groups: 0,
                    };
                    host.draw(&root, &pso, &draw).unwrap()
                });
                let mut wrong = Vec::new();
                for y in 0..SIZE {
                    for x in 0..SIZE {
                        let i = y * SIZE + x;
                        let got: [f32; 4] =
                            std::array::from_fn(|c| drawn[0][i][c] + drawn[1][i][c]);
                        let want = expect(x as f32, y as f32);
                        if got != want {
                            wrong.push((x, y, got, want));
                        }
                    }
                }
                eprintln!("{name} {instruction} {case}: {} wrong / {} pixels", wrong.len(), SIZE * SIZE);
                if let Some(first) = wrong.first() {
                    failures.push(format!(
                        "{name} {case} by {instruction}: {} of {} pixels wrong, first \
                         (x, y, got, expected) {first:?}",
                        wrong.len(),
                        SIZE * SIZE
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
