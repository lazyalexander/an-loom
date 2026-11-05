from contextlib import contextmanager
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest

import remote_compare as probe


class Sessions:
    def __init__(self, events):
        self.stream = events
        self.created = []
        self.sent = []
        self.events = SimpleNamespace(create=lambda *args, **kwargs: self.sent.append((args, kwargs)))

    @contextmanager
    def create(self, **request):
        self.created.append(request)
        yield (SimpleNamespace(to_dict=lambda value=value: value) for value in self.stream)


class RemoteProbe(unittest.TestCase):
    def setup_plan(self, directory):
        fixture = json.loads(probe.FIXTURE.read_text())
        proof = {"schema": "cad-material-evidence-v1", "part": "grip", "material": fixture["material"],
                 "source": {"application": "web-cad", "document": "handwheel",
                            "epoch": "fixture-epoch", "revision": 7},
                 "artifact": {"private_path": "operator-only"}}
        return probe.plan_for(proof, "gpt-6-astra", directory)

    def events(self, plan):
        return [
            {"type": "agent.session.turn.created", "session_id": "sess_1", "turn_id": "turn_1", "turn": {}},
            {"type": "agent.session.requires_action", "session_id": "sess_1", "session": {"required_actions": [
                {"type": "function_call", "turn_id": "turn_1", "call_id": "call_1", "name": probe.TOOL,
                 "arguments": {"fixture": plan["tool_output"]["fixture"]}}]}},
            {"type": "agent.session.turn.output_text.done", "session_id": "sess_1",
             "turn_id": "turn_1", "text": json.dumps(plan["tool_output"])},
            {"type": "agent.session.turn.completed", "session_id": "sess_1", "turn_id": "turn_1",
             "turn": {}, "usage": {"input_tokens": 100, "output_tokens": 50}},
        ]

    def test_exact_approval_and_single_bounded_disclosure(self):
        with TemporaryDirectory() as scratch:
            output = Path(scratch) / "receipt"
            plan = self.setup_plan(output)
            sessions = Sessions(self.events(plan))
            with self.assertRaises(ValueError):
                probe.run_once(sessions, plan, "wrong")
            self.assertFalse(output.exists())
            self.assertEqual(sessions.created, [])
            result = probe.run_once(sessions, plan, probe.digest(plan))
            self.assertEqual(result["usage"]["output_tokens"], 50)
            self.assertEqual(sessions.created[0]["environment"], {"type": "none"})
            self.assertEqual(sessions.created[0]["spend_control"], {"limit": 5})
            self.assertEqual(len(sessions.sent), 1)
            request = sessions.sent[0][1]["events"][0]
            self.assertEqual(json.loads(request["output"]), plan["tool_output"])
            self.assertNotIn("operator-only", json.dumps(sessions.created + sessions.sent))
            with self.assertRaises(FileExistsError):
                probe.run_once(sessions, plan, probe.digest(plan))
            self.assertEqual(len(sessions.created), 1)

    def test_wrong_action_and_unconfirmed_stream_fail_closed(self):
        with TemporaryDirectory() as scratch:
            base = Path(scratch)
            for case in ("wrong", "duplicate", "disconnect", "wrong-proof", "boolean-number"):
                with self.subTest(case=case):
                    output = base / case
                    plan = self.setup_plan(output)
                    events = self.events(plan)
                    if case == "wrong":
                        events[1]["session"]["required_actions"][0]["arguments"]["fixture"] = "another"
                    elif case == "duplicate":
                        events.insert(2, events[1])
                    elif case == "disconnect":
                        events = events[:2]
                    elif case == "wrong-proof":
                        events[2]["text"] = "{}"
                    else:
                        answer = json.loads(events[2]["text"])
                        answer["material"]["metallic"] = False
                        events[2]["text"] = json.dumps(answer)
                    sessions = Sessions(events)
                    with self.assertRaises(RuntimeError):
                        probe.run_once(sessions, plan, probe.digest(plan))
                    self.assertEqual(len(sessions.sent), 0 if case == "wrong" else 1)
                    self.assertFalse((output / "result.json").exists())
                    self.assertEqual((output / "disclosure-intent.json").exists(), case != "wrong")
