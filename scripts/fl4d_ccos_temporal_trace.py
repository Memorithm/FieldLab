#!/usr/bin/env python3
"""Acquire FL-4D temporal evidence from the actual pinned CCOS MCP runtime."""

import argparse
import glob
import json
import math
import os
import shutil
import subprocess
import tempfile

ANCHORS = {
    "A": "src/external_memory.rs",
    "B": "src/agent_session.rs",
    "C": "src/migrate.rs",
    "D": "src/region_metrics.rs",
}
CALIBRATION = list("AAABAAAA") + list("BBBCBBBB") + list("CCCDCCCC")
HOLDOUT = list("CCACCCCC") + list("AADAaaaa".upper()) + list("DDBDDDDD")
EXPECTED_LEN = 24


def load_flat_src(crate_src):
    files = {}
    for path in sorted(glob.glob(os.path.join(crate_src, "*.rs"))):
        with open(path, encoding="utf-8", errors="strict") as handle:
            files["src/" + os.path.basename(path)] = handle.read()
    return files


def tool_call(request_id, name, arguments):
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    }


def result_text(response):
    if "error" in response:
        raise RuntimeError(f"CCOS MCP error: {response['error']}")
    content = response.get("result", {}).get("content", [])
    if not content or "text" not in content[0]:
        raise RuntimeError(f"CCOS MCP response missing text result: {response}")
    return json.loads(content[0]["text"])


def execute_trace(ccos, files, schedule, budget, depth, workdir):
    workspace = tempfile.mkdtemp(prefix="fl4d_", dir=workdir)
    requests = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}]
    request_id = 2

    for uri, source in files.items():
        requests.append(tool_call(request_id, "ingest", {"uri": uri, "source": source}))
        request_id += 1

    verify_id = request_id
    requests.append(tool_call(request_id, "verify", {}))
    request_id += 1

    ids = []
    for step, symbol in enumerate(schedule):
        anchor = ANCHORS[symbol]
        signal_id = request_id
        requests.append(
            tool_call(
                request_id,
                "signal_failure",
                {"node": "file:" + anchor, "depth": depth},
            )
        )
        request_id += 1
        recall_id = request_id
        requests.append(
            tool_call(
                request_id,
                "recall",
                {"strategy": "working_set", "budget": budget},
            )
        )
        request_id += 1
        ids.append((step, symbol, signal_id, recall_id))

    payload = "\n".join(json.dumps(request, separators=(",", ":")) for request in requests) + "\n"
    process = subprocess.run(
        [ccos, "mcp", workspace],
        input=payload,
        capture_output=True,
        text=True,
        timeout=900,
        check=False,
    )
    try:
        if process.returncode != 0:
            raise RuntimeError(
                f"CCOS exited {process.returncode}: stderr={process.stderr[-4000:]}"
            )

        responses = {}
        for line in process.stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                response = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(response.get("id"), int):
                responses[response["id"]] = response

        if verify_id not in responses:
            raise RuntimeError("missing CCOS verify response")
        verify = result_text(responses[verify_id])
        if not verify.get("valid", False):
            raise RuntimeError(f"CCOS integrity verification failed: {verify}")

        observations = []
        for step, symbol, signal_id, recall_id in ids:
            if signal_id not in responses or recall_id not in responses:
                raise RuntimeError(f"missing CCOS response at FL-4D step {step}")
            signal = result_text(responses[signal_id])
            recall = result_text(responses[recall_id])
            items = []
            for item in recall.get("items", []):
                score = float(item["score"])
                if not math.isfinite(score):
                    raise RuntimeError(f"non-finite CCOS score at step {step}")
                items.append(
                    {
                        "uri": str(item["uri"]),
                        "score": score,
                        "kind": str(item["kind"]),
                    }
                )
            observations.append(
                {
                    "step": step,
                    "stimulus": symbol,
                    "anchor": ANCHORS[symbol],
                    "affected": int(signal.get("affected", -1)),
                    "tokens": int(recall.get("tokens", -1)),
                    "items": items,
                }
            )
        return observations
    finally:
        shutil.rmtree(workspace, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("crate_src")
    parser.add_argument("--ccos", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--budget", type=int, default=2048)
    parser.add_argument("--depth", type=int, default=3)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    if len(CALIBRATION) != EXPECTED_LEN or len(HOLDOUT) != EXPECTED_LEN:
        raise RuntimeError("FL-4D frozen schedules must contain exactly 24 observations")
    if not os.path.exists(args.ccos):
        raise SystemExit(f"CCOS binary not found: {args.ccos}")

    files = load_flat_src(args.crate_src)
    missing = [path for path in ANCHORS.values() if path not in files]
    if missing:
        raise RuntimeError(f"pinned FL-4D anchors missing from corpus: {missing}")

    workdir = tempfile.mkdtemp(prefix="fieldlab_fl4d_")
    try:
        report = {
            "experiment": "FL-4D",
            "protocol": "native-temporal-ccos-trace-v1",
            "ccos_commit": args.commit,
            "source_path": args.crate_src,
            "files": len(files),
            "budget": args.budget,
            "depth": args.depth,
            "anchors": ANCHORS,
            "calibration": execute_trace(
                args.ccos, files, CALIBRATION, args.budget, args.depth, workdir
            ),
            "holdout": execute_trace(
                args.ccos, files, HOLDOUT, args.budget, args.depth, workdir
            ),
        }
    finally:
        shutil.rmtree(workdir, ignore_errors=True)

    os.makedirs(os.path.dirname(args.out) or ".", exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, sort_keys=True, indent=2, allow_nan=False)
        handle.write("\n")


if __name__ == "__main__":
    main()
