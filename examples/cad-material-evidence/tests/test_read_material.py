from hashlib import sha256
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
from zipfile import ZipFile

import read_material


class AcceptedMaterial(unittest.TestCase):
    source = {"application": "web-cad", "document": "handwheel", "epoch": "e", "revision": 7}
    finish = {"finish": "walnut", "color": "#4a2c23", "metallic": 0, "roughness": 0.8}
    scene = {"source": source, "drawing": "handwheel", "request_sha256": "a" * 64,
             "materials": {"grip": finish, "rail": {"finish": "other"}}, "solids": ["private"]}

    def test_reads_only_the_exact_approved_part_and_checks_the_payload(self):
        with TemporaryDirectory() as tmp:
            fixture = Path(tmp) / "fixture.ora"
            with ZipFile(fixture, "w") as archive:
                archive.writestr("data/scene.json", json.dumps(self.scene))
            payload = fixture.read_bytes()
            binding = {"revision": "b" * 64, "content_sha256": sha256(payload).hexdigest(),
                       "encoding": "bytes", "bytes": len(payload)}
            report = {"schema": "an-native-selected-artifact-v1", "payload_verified": True,
                      "run": "paint", "instance": "instance", "release": "d" * 64, "catalog": "fixed",
                      "output": {"path": "painting.ora", "binding": binding}}

            def host(_, command, *args):
                if command == "spool-artifacts":
                    return report
                self.assertEqual(args[:2], ("fixed", "b" * 64))
                args[2].write_bytes(payload)
                return {}

            with patch.object(read_material, "host_call", side_effect=host):
                result = read_material.read_accepted("host", "instance", "paint", "d" * 64, "b" * 64,
                                                    "handwheel", "e", 7, "grip")
                self.assertEqual(result["material"], self.finish)
                self.assertEqual(result["artifact"]["revision"], "b" * 64)
                self.assertNotIn("private", json.dumps(result))
                self.assertNotIn("rail", json.dumps(result))
                for revision, document, epoch, source_rev, part in [
                    ("c" * 64, "handwheel", "e", 7, "grip"), ("b" * 64, "vise", "e", 7, "grip"),
                    ("b" * 64, "handwheel", "other", 7, "grip"), ("b" * 64, "handwheel", "e", 8, "grip"),
                    ("b" * 64, "handwheel", "e", 7, "missing")
                ]:
                    with self.assertRaises(ValueError):
                        read_material.read_accepted("host", "instance", "paint", "d" * 64, revision,
                                                    document, epoch, source_rev, part)
                binding["content_sha256"] = "c" * 64
                with self.assertRaisesRegex(ValueError, "differs"):
                    read_material.read_accepted("host", "instance", "paint", "d" * 64, "b" * 64,
                                                "handwheel", "e", 7, "grip")
