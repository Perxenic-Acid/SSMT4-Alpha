---
name: zzmi-gametype-analysis
description: Check whether every IB buffer in a ZZZ 3Dmigoto FrameAnalysis folder can be parsed by the project's existing ZZMI data types (GameType JSON), identify which IB hashes are not parseable, derive their required vertex-buffer data type, and add the missing data type files following the project's conventions. Use when asked to verify IB parseability against ZZMI data types, find unsupported IB layouts in a FrameAnalysis extraction folder, add ZZMI GameType JSON files, or audit/update ZZMI data type coverage after new frame captures.
---

# ZZMI GameType Analysis

## Workflow

1. **Inventory the extraction folder** (K:/SSMT-Package-master/3Dmigoto/ZZZ/FrameAnalysis-<timestamp>):
   - IB txt files match NNNNNN-ib=<8-hex>-vs=<16-hex>-ps=<16-hex>.txt.
   - Top-level .buf/.txt entries are 0-byte shortcuts; real content lives in deduped/ and is resolved through the log.txt mapping.
   - Deduplicate by the 8-hex IB hash: parse topology and format (all files are DXGI_FORMAT_R16_UINT in practice).

2. **Run the checker script** from the project root:

        python <skill>/scripts/zzmi_ib_check.py "K:/SSMT-Package-master/3Dmigoto/ZZZ/FrameAnalysis-<timestamp>" --json report.json

   The script replicates get_possible_gametype_list_unity_vs (see [PROJECT-CONVENTIONS.md](references/PROJECT-CONVENTIONS.md)) and reports for each unique IB hash: pointlist index, trianglelist index list, and matched GameType names.

3. **Interpret results**:
   - Every IB hash has at least one matched type -> all IBs are parseable; no changes needed.
   - Any IB hash shows matched=[] -> that IB layout is unsupported. Investigate by opening the paired NNNNNN-vbN=... txt headers of its trianglelist/pointlist draws (element semantic/format/InputSlot/stride) and derive the required data type.

4. **Add missing data types** (one file per data type, project convention):
   - Edit src-tauri/src/gametype/type_zzmi.rs: append a D3D11GameType::from_parts("<TYPE_NAME>", vec![...]) block using the constants from src-tauri/src/constants/gametype_*.rs (ElementName, DxgiFormat, ExtractSlot, ExtractTechnique, CategoryName).
   - Regenerate JSON: run node scripts/export-gametype-json.mjs from the repo root (writes one JSON per type into src-tauri/resources/GameType/ZZMI/). Never hand-edit the JSON; it is generated from the Rust sources.
   - Type name encodes the layout, e.g. GPU_P12_N12_TA16_C16_T8_T1-8_T2-8_T3-8_BW16_BI16_ (see the references file for the naming table).

5. **Verify**:
   - Re-run step 2; every IB hash must now match, and previously matched hashes must keep the same matches (no regressions).
   - Snapshot hashes of src-tauri/resources/GameType/** before exporting, and confirm only the intended new JSON files appear.
   - If Rust is available, run cargo check in src-tauri/.

## Pitfalls

- Always resolve frame filenames through the log.txt "Dumping Buffer ... -> deduped\..." map; top-level files are zero-byte symlinks.
- GPU types use the pointlist-index buffer for Position/Texcoord when a pointlist index exists; the pointlist index is derived from the log (IASetVertexBuffers slot 0 hash, last matching draw before the first trianglelist index).
- CPU types compute file size from the txt stride x vertex count; GPU types use the buf file size.
- A type may legitimately match multiple IB hashes, and one IB hash may match several GPU types (the extractor keeps all of them).
- The export script regenerates every GameType JSON; commit the Rust source and rerun it rather than patching JSON by hand.

## Resources

- scripts/zzmi_ib_check.py - the match checker (Python 3, stdlib only).
- references/PROJECT-CONVENTIONS.md - type-name encoding, JSON format, matching algorithm details, and add-type checklist.
