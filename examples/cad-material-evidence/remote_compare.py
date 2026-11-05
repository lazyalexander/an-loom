#!/usr/bin/env python3
"""Compare a local CAD fixture proof with one OpenAI Agents API turn."""
import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import time

ENDPOINT = "https://api.openai.com/v1"
TIME_LIMIT = 45
SPEND_LIMIT_CENTS = 5
TOOL = "read_approved_grip"
FIXTURE = Path(__file__).with_name("public-grip.json")
INSTRUCTIONS = (
    "Call read_approved_grip exactly once with the public fixture identifier from the request. "
    "Then return exactly the tool's JSON as a single JSON object, with no Markdown or commentary."
)


def encoded(value):
    return json.dumps(value, ensure_ascii=True, allow_nan=False, sort_keys=True,
                      separators=(",", ":")).encode("ascii")


def digest(value):
    return sha256(encoded(value)).hexdigest()


def plan_for(proof, model, directory):
    fixture = json.loads(FIXTURE.read_text())
    if (proof.get("schema") != "cad-material-evidence-v1" or proof.get("part") != fixture["part"]
            or proof.get("source") != {"application": "web-cad", "document": fixture["document"],
                                       "epoch": "fixture-epoch", "revision": fixture["revision"]}
            or proof.get("material") != fixture["material"]):
        raise ValueError("Expected the local qualification proof for the public grip fixture")
    if not re.fullmatch(r"[a-z][a-z0-9.-]{1,63}", model):
        raise ValueError("Expected an explicit OpenAI model identifier")
    evidence = {**fixture, "citation": "sha256:" + digest(fixture)}
    tool = {"type": "function", "name": TOOL, "description": "Read the approved public CAD material record.",
            "parameters": {"type": "object", "properties": {"fixture": {"type": "string"}},
                           "required": ["fixture"], "additionalProperties": False}}
    request = {"agent": {"model": model, "instructions": INSTRUCTIONS, "tools": [tool]},
               "environment": {"type": "none"},
               "spend_control": {"limit": SPEND_LIMIT_CENTS},
               "input": [{"role": "user", "content": [{"type": "input_text",
                          "text": f"Verify the grip material in fixture {fixture['fixture']}."}]}]}
    return {"schema": "cad-remote-fixture-plan-v1", "endpoint": ENDPOINT,
            "output": str(directory), "request": request, "tool_output": evidence,
            "max_turns": 1, "max_tool_results": 1, "observation_seconds": TIME_LIMIT}


def receipt(directory, name, value):
    descriptor = os.open(directory / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as target:
        target.write(encoded(value) + b"\n")
        target.flush()
        os.fsync(target.fileno())


def run_once(sessions, plan, approval):
    if approval != digest(plan):
        raise ValueError("Approval must match the exact remote disclosure plan")
    directory = Path(plan["output"])
    directory.mkdir(mode=0o700)
    receipt(directory, "intent.json", {"approval_sha256": approval, "plan": plan})
    start = time.monotonic()
    session_id = turn_id = sent_call = None
    answer = None
    action_at = submitted_at = None
    with sessions.create(**plan["request"], stream=True) as events:
        for raw in events:
            if time.monotonic() - start > TIME_LIMIT:
                raise RuntimeError("Observation deadline exceeded; inspect the saved session")
            event = raw.to_dict()
            kind = event["type"]
            observed = event.get("session_id")
            if observed:
                if session_id is None:
                    session_id = observed
                    receipt(directory, "session.json", {"session_id": session_id})
                elif observed != session_id:
                    raise RuntimeError("Unexpected session identity")
            if kind == "agent.session.turn.created":
                if turn_id is not None or event["turn"].get("subagent_id"):
                    raise RuntimeError("Expected one root turn")
                turn_id = event["turn_id"]
                receipt(directory, "turn.json", {"session_id": session_id, "turn_id": turn_id})
            elif kind == "agent.session.requires_action":
                actions = event["session"]["required_actions"]
                if not session_id or not turn_id or sent_call or len(actions) != 1:
                    raise RuntimeError("Unexpected number of pending actions")
                action = actions[0]
                if (action["type"] != "function_call" or action["name"] != TOOL
                        or action["turn_id"] != turn_id or not action["call_id"]
                        or action["arguments"] != {"fixture": plan["tool_output"]["fixture"]}):
                    raise RuntimeError("Requested action exceeds the approved fixture scope")
                action_at = time.monotonic()
                sent_call = action["call_id"]
                receipt(directory, "disclosure-intent.json", {"session_id": session_id, "turn_id": turn_id,
                    "call_id": sent_call, "output_sha256": digest(plan["tool_output"])})
                sessions.events.create(session_id, idempotency_key=approval, events=[{
                    "type": "agent.session.input.tool_result", "turn_id": turn_id, "call_id": sent_call,
                    "success": True, "output": encoded(plan["tool_output"]).decode("ascii")}])
                submitted_at = time.monotonic()
            elif kind == "agent.session.turn.output_text.done" and sent_call:
                if event.get("turn_id") != turn_id or answer is not None or len(event["text"]) > 2048:
                    raise RuntimeError("Unexpected proof output")
                answer = json.loads(event["text"])
            elif kind in ("agent.session.turn.failed", "agent.session.turn.cancelled",
                          "agent.session.failed", "agent.session.environment.failed", "error"):
                raise RuntimeError(f"Remote operation ended with {kind}")
            elif kind == "agent.session.turn.completed":
                if (not sent_call or event["turn_id"] != turn_id or event["turn"].get("subagent_id")
                        or answer != plan["tool_output"] or type(answer["revision"]) is not int
                        or any(type(answer["material"][key]) not in (int, float)
                               for key in ("metallic", "roughness"))):
                    raise RuntimeError("Remote result does not match the local fixture proof")
                completed_at = time.monotonic()
                result = {"matched": True, "session_id": session_id, "turn_id": turn_id,
                          "call_id": sent_call, "elapsed_ms": round((time.monotonic() - start) * 1000),
                          "usage": event.get("usage"), "evidence": answer}
                receipt(directory, "result.json", result)
                return result
    raise RuntimeError("Stream ended before confirmation; inspect the saved session")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--local-proof", type=Path, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--approve")
    args = parser.parse_args()
    plan = plan_for(json.loads(args.local_proof.read_text()), args.model, args.output.resolve())
    approval = digest(plan)
    if args.approve is None:
        print(json.dumps({"approval_sha256": approval, "plan": plan}, indent=2))
        return
    if args.approve != approval:
        parser.error("Approval does not match the exact remote disclosure plan")
    if not os.environ.get("OPENAI_API_KEY"):
        parser.error("OPENAI_API_KEY is required for live qualification")
    from openai import OpenAI
    with OpenAI(base_url=ENDPOINT, max_retries=0, timeout=10) as client:
        print(json.dumps(run_once(client.beta.agents.sessions, plan, args.approve), indent=2))


if __name__ == "__main__":
    main()
