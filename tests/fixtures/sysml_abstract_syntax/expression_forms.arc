// Every form of value, expression and multiplicity the SysML v2 export can
// write, for the
// comparison with the OMG pilot (tools/sysml_abstract_syntax_check.py).
model ExpressionForms {}

type "Task" {
    period: 20 ms
    gain: 1.5
}

system_analysis "Forms" {
    function "Sample" {
        id: "SF-A"
        is: "Task"
        period: 10 ms
        wcet: 0.004 s
        offset: -5
        gain: 2.5
        count: 3
        label: "a \"quoted\" label"
        mass: 12 kg
        speed: 130 km/h
        load: 40 %
        energy: 2 kWh
        memory: 512 MiB
        rate: 9600 bps
        bandwidth: 2 Mbps
        clock: 16 MHz
        pressure: 3 bar
        supply: 12 V
        distance: 250 m
    }
    function "Filter" {
        id: "SF-B"
        is: "Task"
        wcet: 6 ms
    }
    functional_chain "Acquire" {
        id: "FC-A"
        involves: ["SF-A", "SF-B"]
        latency_budget: 40 ms
    }
}

logical_architecture "Multiplicities" {
    component "Sensor" {
        id: "LC-SNS"
        multiplicity: 2
        component "Lens" { id: "LC-LNS" multiplicity: 4 }
    }
    component "Backup" { id: "LC-BKP" multiplicity: "0..1" }
    component "Channel" { id: "LC-CHN" multiplicity: "1..*" }
    component "Probe" { id: "LC-PRB" multiplicity: "*" }
    component "Hub" { id: "LC-HUB" }
}

physical_architecture "Hardware" {
    node "Compute unit" { id: "PN-CPU" multiplicity: "2..3" }
}

constraint "Fastest stage" {
    id: "CST-MIN"
    assert: min("FC-A", wcet) >= 1 ms
}

constraint "Slowest stage" {
    id: "CST-MAX"
    assert: max("FC-A", wcet) + 2 ms < "FC-A".latency_budget / 2
}

constraint "Total" {
    id: "CST-SUM"
    assert: sum("FC-A", wcet) - "SF-B".wcet * 2 <= "SF-A".period
}

constraint "Stages" {
    id: "CST-CNT"
    assert: count("FC-A") == 2
}
