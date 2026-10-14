"""Offline consistency checks using an already installed JSON Schema validator."""
import json
from pathlib import Path
from copy import deepcopy
from jsonschema import Draft202012Validator

root = Path(__file__).resolve().parent.parent
schema = json.loads((root / "docs/description-0.2.schema.json").read_text(encoding="utf-8"))
dashboard = json.loads((root / "examples/dashboard.json").read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
validator.validate(dashboard)
validator.validate(json.loads((root / "examples/command.json").read_text(encoding="utf-8")))
step_button = json.loads((root / "examples/step-button.json").read_text(encoding="utf-8"))
validator.validate(step_button)
scoped = json.loads((root / "examples/scoped-state.json").read_text(encoding="utf-8"))
validator.validate(scoped)
modules = json.loads((root / "examples/modules.json").read_text(encoding="utf-8"))
validator.validate(modules)
linkage = json.loads((root / "examples/module-linkage.json").read_text(encoding="utf-8"))
validator.validate(linkage)
validator.validate(json.loads((root / "examples/plugin-chart.json").read_text(encoding="utf-8")))
http_app = json.loads((root / "examples/http-chart.json").read_text(encoding="utf-8"))
validator.validate(http_app)
auth_app = json.loads((root / "examples/http-auth.json").read_text(encoding="utf-8"))
validator.validate(auth_app)
for value in [{"kind":"credential","name":"../escape"}, {"kind":"credential","name":"token","value":"forbidden"}]:
    invalid = deepcopy(auth_app)
    invalid["dataSources"]["protected"]["headers"]["Authorization"] = value
    assert not validator.is_valid(invalid)
validator.validate(json.loads((root / "examples/plugin-test.json").read_text(encoding="utf-8")))
validator.validate(json.loads((root / "examples/plugin-dashboard.json").read_text(encoding="utf-8")))
for field, value in [("method", "TRACE"), ("intervalMs", 0), ("timeoutMs", 0), ("unknown", True)]:
    invalid = deepcopy(http_app)
    invalid["dataSources"]["metrics"][field] = value
    assert not validator.is_valid(invalid)
for binding in [{"kind": "prop", "key": "message", "path": "bad"}, {"kind": "event", "path": "/~2"}]:
    invalid = deepcopy(linkage)
    invalid["root"]["actions"]["receiveMessage"]["steps"][0]["value"] = binding
    assert not validator.is_valid(invalid)
invalid = deepcopy(linkage)
invalid["root"]["children"][0]["props"] = {}
assert not validator.is_valid(invalid)
for field, value in [("title", {"kind": "literal", "value": "bad"}), ("events", {"click": "next"})]:
    invalid = deepcopy(modules)
    invalid["root"]["children"][1][field] = value
    assert not validator.is_valid(invalid)
invalid = deepcopy(scoped)
invalid["root"]["children"][0]["state"] = {"forbidden": 1}
assert not validator.is_valid(invalid)
for event_action in [42, {"kind": "state", "key": "wizard", "extra": 1}]:
    invalid = deepcopy(step_button)
    invalid["root"]["children"][1]["events"]["click"] = event_action
    assert not validator.is_valid(invalid)
for field in ["bogus", "rows", "span"]:
    invalid = deepcopy(dashboard)
    invalid["root"][field] = 1
    assert not validator.is_valid(invalid), field
invalid = deepcopy(dashboard)
invalid["root"]["children"][2]["columns"][0]["fr"] = 0
assert not validator.is_valid(invalid)
print("PASS draft 2020-12 schema, all eleven examples and invalid field/track/event/scope/module/communication/HTTP/credential cases")
