#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ZZMI FrameAnalysis IB -> GameType matching checker.

Replicates src-tauri/src/extract_new/zzmi.rs get_possible_gametype_list_unity_vs:
  1. filter_trianglelist_index_unity_vs: pick first trianglelist index where every
     trianglelist category slot has a .buf frame file.
  2. For each category: use pointlist index (if topology=pointlist and found via log)
     else trianglelist index; locate slot .buf/.txt frame file, map through log dedup,
     compute file size (buf size for GPU, stride*vertex_count from txt for CPU).
  3. Vertex count consistency: size / category_stride must be equal across categories,
     non-zero, remainder 0 for CPU.
  4. GPU Texcoord extra check: compare per-element byte widths in pointlist txt; fallback
     compares total length + element count.
  5. Single-category CPU extra check: txt stride must equal GameType total stride.

Usage:
  python zzmi_ib_check.py <frame_analysis_root> [--gametype-dir <dir>] [--verbose] [--json <out>]
"""
import argparse
import collections
import json
import os


def bw_from_format(fmt: str) -> int:
    f = fmt.lower()
    total = 0
    for i in range(len(f) - 2):
        w = f[i:i + 3]
        if w[1] == "8":
            total += 1
        elif w[1] == "3" and w[2] == "2":
            total += 4
        elif w[1] == "1" and w[2] == "6":
            total += 2
    return total


def build_index(files):
    return sorted(set(files))


def first_file(files, content, suffix):
    for f in files:
        if content in f and f.endswith(suffix):
            return f
    return ""


def build_deduped_map(lines):
    out = {}
    for line in lines:
        if "->" not in line:
            continue
        start = None
        if "Dumping Texture2D" in line:
            start = line.find("Dumping Texture2D") + len("Dumping Texture2D")
        elif "Dumping Buffer" in line:
            start = line.find("Dumping Buffer") + len("Dumping Buffer")
        else:
            continue
        splits = line[start:].split("->")
        if len(splits) < 2:
            continue
        orig = os.path.basename(splits[-2].strip())
        ded = os.path.basename(splits[-1].strip())
        if orig.endswith(".lnk"):
            orig = orig[:-4]
        if ded.endswith(".lnk"):
            ded = ded[:-4]
        out[orig] = ded
    return out


def deduped_path(root, deduped_map, name):
    d = deduped_map.get(name, "")
    return os.path.join(root, "deduped", d) if d else ""


def load_types(gametype_dir):
    types = []
    for jp in sorted(os.listdir(gametype_dir)):
        if not jp.endswith(".json"):
            continue
        name = jp[:-5]
        data = json.load(open(os.path.join(gametype_dir, jp), encoding="utf-8"))
        categories = []
        cat_slot = {}
        cat_topo = {}
        cat_stride = collections.Counter()
        elem_dict = {}
        counts = collections.Counter()
        for e in data["D3D11ElementList"]:
            sem = e["SemanticName"]
            idx = counts[sem]
            counts[sem] += 1
            el_name = sem if idx == 0 else f"{sem}{idx}"
            cat = e["Category"]
            if cat not in categories:
                categories.append(cat)
            cat_slot[cat] = e["ExtractSlot"]
            cat_topo[cat] = e["ExtractTechnique"]
            cat_stride[cat] += int(e["ByteWidth"])
            elem_dict[el_name] = {"category": cat, "byte_width": int(e["ByteWidth"])}
        gpu = any(e["SemanticName"] == "BLENDINDICES" for e in data["D3D11ElementList"]) or any(
            e["ExtractTechnique"] == "pointlist" for e in data["D3D11ElementList"]
        )
        types.append(
            {
                "name": name,
                "gpu": gpu,
                "categories": categories,
                "cat_slot": cat_slot,
                "cat_topo": cat_topo,
                "cat_stride": cat_stride,
                "elem_dict": elem_dict,
                "self_stride": sum(cat_stride[c] for c in categories),
            }
        )
    types.sort(key=lambda t: (not t["gpu"], t["name"]))
    return types


def get_trianglelist_index_list(files, draw_ib):
    idx_set = set()
    for f in files:
        if f"-ib={draw_ib}" in f and f.endswith(".txt"):
            idx_set.add(f[:6])
    if not idx_set:
        for f in files:
            if f"-ib={draw_ib}" in f and f.endswith(".buf"):
                idx_set.add(f[:6])
    return sorted(idx_set)


def get_all_drawib_list(files):
    out = set()
    for f in files:
        if "-ib=" in f and f.endswith(".txt"):
            h = f[10:18]
            if len(h) == 8:
                out.add(h)
    return sorted(out)


def get_drawcall_index_list_by_hash(lines, draw_ib):
    index_list = []
    current = ""
    for line in lines:
        if line.startswith("00") and len(line) >= 6:
            current = line[:6]
        if f"hash={draw_ib}" in line:
            if current not in index_list:
                index_list.append(current)
    return index_list


def get_line_list_by_index(lines, index):
    try:
        index_number = int(index)
    except ValueError:
        return []
    out = []
    find = False
    for line in lines:
        if line.startswith("00") and not find:
            sub = line[:6]
            if sub.isdigit() and int(sub) == index_number:
                find = True
                out.append(line)
                continue
        if find:
            if line.startswith("00"):
                sub = line[:6]
                if sub.isdigit():
                    if int(sub) > index_number:
                        break
                    out.append(line)
                else:
                    out.append(line)
            else:
                out.append(line)
    return out


def get_last_pointlist_index_by_hash(lines, draw_ib):
    dl = get_drawcall_index_list_by_hash(lines, draw_ib)
    if not dl:
        return None
    first_tl = dl[0]
    line_list = get_line_list_by_index(lines, first_tl)
    vb0_hash = ""
    find_ia = False
    for call_line in line_list:
        if "IASetVertexBuffers" in call_line and not find_ia:
            find_ia = True
            continue
        if find_ia:
            if not call_line.startswith("00"):
                parts = call_line.split(":", 1)
                if len(parts) == 2 and parts[0].strip() == "0":
                    for kv in parts[1].split():
                        if kv.startswith("hash="):
                            vb0_hash = kv[5:]
                            break
            else:
                break
    if not vb0_hash:
        return None
    current = ""
    possible = []
    try:
        tl_num = int(first_tl)
    except ValueError:
        return None
    for log_line in lines:
        if log_line.startswith("00"):
            current = log_line[:6]
        if f"hash={vb0_hash}" in log_line and "dst" not in log_line.lower():
            try:
                pl = int(current)
            except ValueError:
                continue
            if pl < tl_num and current not in possible:
                possible.append(current)
    return possible[-1] if possible else None


def vb_txt_elements(path):
    elements = []
    cur = None
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")
            if line.startswith("element["):
                if cur and "SemanticName" in cur:
                    elements.append(cur)
                cur = {}
            elif cur is not None:
                t = line.strip()
                tl = t.lower()
                if tl.startswith("semanticname:"):
                    cur["SemanticName"] = t.split(":", 1)[1].strip()
                elif tl.startswith("semanticindex:"):
                    cur["SemanticIndex"] = int(t.split(":", 1)[1].strip())
                elif tl.startswith("format:"):
                    cur["Format"] = t.split(":", 1)[1].strip()
                elif not tl.startswith(("semantic", "format", "inputslot", "aligned", "instancedata", "inputslo")):
                    if "SemanticName" in cur:
                        elements.append(cur)
                    cur = None
    if cur and "SemanticName" in cur:
        elements.append(cur)
    elem_dict = {}
    for e in elements:
        idx = e.get("SemanticIndex", 0)
        en = e["SemanticName"] if idx == 0 else f"{e['SemanticName']}{idx}"
        elem_dict[en] = bw_from_format(e["Format"])
    show = []
    cnt = 0
    meet = False
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            if cnt > 10:
                break
            if not meet:
                if line.strip().startswith("vertex-data:"):
                    meet = True
                continue
            t = line.strip()
            if not t.lower().startswith("vb"):
                continue
            cnt += 1
            left = t.split(":", 1)[0]
            toks = left.split()
            if len(toks) >= 2:
                show.append(toks[1].strip())
    return elem_dict, show


def match_type(gt, files, root, deduped_map, lines, draw_ib, pointlist_index, tl_list):
    tl_index = None
    for idx in tl_list:
        ok = True
        for cat, topo in gt["cat_topo"].items():
            if topo != "trianglelist":
                continue
            if not first_file(files, f"{idx}-{gt['cat_slot'][cat]}", ".buf"):
                ok = False
                break
        if ok:
            tl_index = idx
            break
    if tl_index is None:
        return False
    cat_sizes = {}
    for cat in gt["categories"]:
        topo = gt["cat_topo"][cat]
        ex = pointlist_index if (topo == "pointlist" and pointlist_index) else tl_index
        slot = gt["cat_slot"][cat]
        buf_name = first_file(files, f"{ex}-{slot}", ".buf")
        txt_name = first_file(files, f"{ex}-{slot}", ".txt")
        buf_path = deduped_path(root, deduped_map, buf_name)
        txt_path = deduped_path(root, deduped_map, txt_name)
        if not buf_path or not os.path.exists(buf_path):
            return False
        if txt_name and not txt_path:
            return False
        if not txt_name or not txt_path:
            fs = os.path.getsize(buf_path)
        else:
            if gt["gpu"]:
                fs = os.path.getsize(buf_path)
            else:
                md = {}
                for line in open(txt_path, "r", encoding="utf-8", errors="replace"):
                    if line.startswith("stride:"):
                        md["stride"] = int(line[7:].strip())
                    elif line.startswith("vertex count:"):
                        md["vc"] = int(line[13:].strip())
                fs = md.get("stride", 0) * md.get("vc", 0)
        cat_sizes[cat] = fs
    vn = 0
    for cat in gt["categories"]:
        cs = gt["cat_stride"][cat]
        fs = cat_sizes[cat]
        tmp = fs // cs if cs > 0 else 0
        if tmp == 0:
            return False
        if not gt["gpu"] and fs % cs != 0:
            return False
        if vn == 0:
            vn = tmp
        elif vn != tmp:
            return False
        else:
            if gt["gpu"] and cat == "Texcoord" and pointlist_index:
                slot = gt["cat_slot"][cat]
                txt_name = first_file(files, f"{pointlist_index}-{slot}", ".txt")
                if not txt_name:
                    return False
                txt_path = deduped_path(root, deduped_map, txt_name)
                if not txt_path or not os.path.exists(txt_path):
                    return False
                elem_dict, show = vb_txt_elements(txt_path)
                all_bw = True
                for en, el in gt["elem_dict"].items():
                    if el["category"] != "Texcoord":
                        continue
                    if en in elem_dict:
                        if el["byte_width"] != elem_dict[en]:
                            all_bw = False
                            break
                if not all_bw:
                    txt_len = sum(elem_dict.get(en, 0) for en in show)
                    txt_num = len(show)
                    gt_len = sum(el["byte_width"] for el in gt["elem_dict"].values() if el["category"] == "Texcoord")
                    gt_num = sum(1 for el in gt["elem_dict"].values() if el["category"] == "Texcoord")
                    if txt_len != gt_len or txt_num != gt_num:
                        return False
    if not gt["gpu"] and len(gt["cat_slot"]) == 1:
        slot = gt["cat_slot"][gt["categories"][0]]
        txt_name = first_file(files, f"{tl_index}-{slot}", ".txt")
        if not txt_name:
            return False
        txt_path = deduped_path(root, deduped_map, txt_name)
        if not txt_path or not os.path.exists(txt_path):
            return False
        ss = ""
        for line in open(txt_path, "r", encoding="utf-8", errors="replace"):
            t = line.strip()
            if ":" in t and t.split(":", 1)[0].strip().lower() == "stride":
                ss = t.split(":", 1)[1].strip()
                break
        if ss.strip() and int(ss) != gt["self_stride"]:
            return False
    return True


def get_possible(types, files, root, deduped_map, lines, draw_ib):
    pl = get_last_pointlist_index_by_hash(lines, draw_ib)
    tl = get_trianglelist_index_list(files, draw_ib)
    possible = []
    found = False
    for gt in types:
        if found and not gt["gpu"]:
            continue
        if match_type(gt, files, root, deduped_map, lines, draw_ib, pl, tl):
            possible.append(gt["name"])
            if gt["gpu"]:
                found = True
    all_cpu = not any(t["gpu"] for t in types if t["name"] in possible)
    if all_cpu and possible:
        mx = max(len(next(t for t in types if t["name"] == n)["cat_slot"]) for n in possible)
        possible = [n for n in possible if len(next(t for t in types if t["name"] == n)["cat_slot"]) == mx]
    return pl, tl, possible


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("root")
    parser.add_argument("--gametype-dir", default="src-tauri/resources/GameType/ZZMI")
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--json", default=None)
    args = parser.parse_args()

    root = args.root
    files = sorted(set(os.listdir(root)))
    lines = open(os.path.join(root, "log.txt"), "r", encoding="utf-8", errors="replace").read().splitlines()
    deduped_map = build_deduped_map(lines)
    types = load_types(args.gametype_dir)

    results = []
    for ib in get_all_drawib_list(files):
        pl, tl, poss = get_possible(types, files, root, deduped_map, lines, ib)
        results.append({"ib": ib, "pointlist_index": pl, "trianglelist_indices": tl, "matched": poss})
        print(f"IB {ib}: pointlist={pl} trianglelist={len(tl)} matched={poss if poss else 'NONE'}")
    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump(results, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
