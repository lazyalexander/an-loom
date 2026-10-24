"""Domain rules for a small synthetic service report, independent of any host."""
import json
import math
import re
from pathlib import Path
import zipfile


def aggregate(path):
    source = Path(path).read_bytes()
    if len(source) > 1024 * 1024:
        raise ValueError("Log input exceeds 1 MiB")
    groups = {}
    for line in source.decode().splitlines():
        item = json.loads(line)
        if set(item) != {"service", "status", "latency_ms"}:
            raise ValueError("Unexpected log fields")
        service, status, latency = item["service"], item["status"], item["latency_ms"]
        if not re.fullmatch(r"[a-z][a-z0-9-]{0,31}", service):
            raise ValueError("Invalid service")
        if type(status) is not int or not 100 <= status <= 599:
            raise ValueError("Invalid status")
        if type(latency) not in (int, float) or not math.isfinite(latency) or not 0 <= latency <= 600000:
            raise ValueError("Invalid latency")
        values = groups.setdefault(service, [])
        values.append((status, latency))
    if not 1 <= len(groups) <= 20:
        raise ValueError("Expected between one and twenty services")
    return [{"service": name, "requests": len(values), "errors": sum(s >= 500 for s, _ in values),
             "p95_ms": sorted(v for _, v in values)[math.ceil(.95 * len(values)) - 1],
             "owner": "", "note": ""} for name, values in sorted(groups.items())]


def check_package(path):
    """Consumer-local OOXML limits; the host does not parse document internals."""
    if Path(path).stat().st_size > 8 * 1024 * 1024:
        raise ValueError("Document exceeds 8 MiB")
    with zipfile.ZipFile(path) as package:
        items = package.infolist()
        if len(items) > 512 or sum(i.file_size for i in items) > 32 * 1024 * 1024:
            raise ValueError("Document expansion exceeds limits")
        names = [i.filename for i in items]
        if len(set(names)) != len(names):
            raise ValueError("Duplicate document member")
        for item in items:
            name = item.filename
            if name.startswith("/") or "\\" in name or ".." in name.split("/") or item.flag_bits & 1:
                raise ValueError("Unsafe document member")
            if "vbaproject" in name.lower() or "/externalLinks/" in name or "/embeddings/" in name:
                raise ValueError("Active or external workbook content is not supported")


def validate_rows(rows):
    if not 1 <= len(rows) <= 20:
        raise ValueError("Unexpected report size")
    for row in rows:
        for key in ["requests", "errors"]:
            if type(row[key]) is not int or not 0 <= row[key] <= 1000000:
                raise ValueError("Invalid report count")
        if row["errors"] > row["requests"]:
            raise ValueError("Errors exceed requests")
        if type(row["p95_ms"]) not in (int, float) or not math.isfinite(row["p95_ms"]) or not 0 <= row["p95_ms"] <= 600000:
            raise ValueError("Invalid report latency")
        for key, limit in [("service", 32), ("owner", 60), ("note", 180)]:
            if not isinstance(row[key], str) or len(row[key]) > limit or row[key].startswith("="):
                raise ValueError("Unsupported report text or formula")
    if len({r["service"] for r in rows}) != len(rows):
        raise ValueError("Duplicate service")
    return rows
