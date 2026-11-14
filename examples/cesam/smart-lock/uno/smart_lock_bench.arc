// The smart lock on the bench: the model of ../smart_lock.arc as it is, and
// one more test case, executed at fixed step, that `arclang sil` runs on
// this computer, `arclang pil` in an emulator and `arclang hil` on the
// Arduino Uno holding the generated C (see README.md).
//
//   arclang sil examples/cesam/smart-lock/uno/smart_lock_bench.arc
//   arclang hil examples/cesam/smart-lock/uno/smart_lock_bench.arc --port /dev/cu.usbmodem21101
//   arclang table examples/cesam/smart-lock/uno/smart_lock_bench.arc --kind requirements_coverage -f md
//
// The lock's own model keeps its declared verdicts and its gate; this file
// adds the evidence of a run, kept beside it in .arclang/sim-cache/.

import "../smart_lock.arc"

test_case "TC-7" {
    verifies: ["FUN-1", "TEC-1"]
    method: "test"
    description: "Credential to bolt thrown, executed: the modes machine stepped every 10 ms, a credential at once, SF-1 at 20 ms, the door closed at 100 ms, SF-2 at 150 ms"
    execute: {
        machine: "Lock modes"
        step: 10 ms
        stop_time: 300 ms
        inputs: ["0 s: credential accepted", "20 ms: SF-1", "100 ms: door closed", "150 ms: SF-2"]
    }
    expects: [
        "FUN-1: at(30 ms, \"Lock modes\".Unlocked) == 1",
        "TEC-1: at(160 ms, \"Lock modes\".Locked) == 1",
        "final(\"Lock modes\".Locked) == 1"
    ]
}
