// AI accelerator platform: a compute tray of eight AI accelerators in a
// data-center rack, modeled end to end in Arcadia. Operational need, system
// functions and chains with latency budgets, logical components down to the
// tensor cores and the HBM stacks, physical nodes and links with their
// protocols and data rates, configuration items, a data model, modes,
// scenarios, a HARA with an FMEA, a cybersecurity analysis, requirements
// traced and verified. Compiles with zero findings and passes the production
// gate (ISO 26262, DO-178C and IEC 61508 readings).
//
// Units are those of the metamodel: data rates in Gbps (4.8 TB/s is
// 38400 Gbps), temperatures in kelvin (35 °C is 308 K), energy in kWh.

model AiAcceleratorPlatform {
    name: "AI Accelerator Platform"
    description: "Eight-accelerator compute tray for training and inference at data-center scale: tensor cores, HBM3e stacks, a GPU-to-GPU fabric, a host PCIe/CXL link, a management controller with firmware attestation, liquid cooling"
    version: "1.0"
}

// ---------------------------------------------------------------------------
// Reusable types
// ---------------------------------------------------------------------------

type "Periodic monitor" {
    required: ["latency", "period"]
}

type "Fabric link" {
    bandwidth: 7200 Gbps
    latency: 2 us
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

enumeration "Precision" { id: "EN-PRECISION" values: ["FP8", "BF16", "FP16", "TF32", "FP32", "FP64"] }

enumeration "PowerState" { id: "EN-POWER-STATE" values: ["OFF", "BOOT", "IDLE", "COMPUTE", "THROTTLED", "FAULT"] }

enumeration "CollectiveKind" { id: "EN-COLLECTIVE" values: ["ALL_REDUCE", "ALL_GATHER", "REDUCE_SCATTER", "BROADCAST"] }

data_type "Temperature" { id: "DT-TEMPERATURE" base: "Real" unit: "K" }

data_type "ElectricalPower" { id: "DT-POWER" base: "Real" unit: "W" }

class "TensorDescriptor" {
    id: "CL-TENSOR"
    description: "Shape, layout and precision of a tensor resident in device memory"
    shape: "Integer[]"
    strides: "Integer[]"
    precision: "Precision"
    device_address: "UInt64"
    byte_length: "UInt64"
}

class "KernelLaunch" {
    id: "CL-KERNEL-LAUNCH"
    description: "One kernel enqueued on a device stream"
    kernel_id: "UInt32"
    stream_id: "UInt16"
    grid: "Integer[3]"
    block: "Integer[3]"
    operands: "TensorDescriptor"
}

class "CollectiveOperation" {
    id: "CL-COLLECTIVE"
    description: "A collective over the fabric, one buffer per participating device"
    kind: "CollectiveKind"
    buffer: "TensorDescriptor"
    participants: "UInt8"
    ring_rank: "UInt8"
}

class "TelemetrySample" {
    id: "CL-TELEMETRY"
    description: "One sample per device per second"
    timestamp_ns: "UInt64"
    device_id: "UInt8"
    junction_temperature: "Temperature"
    hbm_temperature: "Temperature"
    board_power: "ElectricalPower"
    sm_utilization_percent: "UInt8"
    hbm_utilization_percent: "UInt8"
    ecc_corrected_count: "UInt32"
    ecc_uncorrected_count: "UInt32"
    power_state: "PowerState"
}

class "ThermalEvent" {
    id: "CL-THERMAL-EVENT"
    description: "Threshold crossing reported by the thermal manager"
    timestamp_ns: "UInt64"
    device_id: "UInt8"
    junction_temperature: "Temperature"
    clock_cap_mhz: "UInt16"
    power_state: "PowerState"
}

class "AttestationReport" {
    id: "CL-ATTESTATION"
    description: "Signed measurement of the device firmware taken at boot"
    device_id: "UInt8"
    firmware_version: "String"
    measurement_sha384: "Bytes"
    nonce: "Bytes"
    signature_ecdsa_p384: "Bytes"
    certificate_chain: "Bytes"
}

association "launches with" { id: "AS-LAUNCH-TENSOR" from: "CL-KERNEL-LAUNCH" to: "CL-TENSOR" role: "operands" kind: "aggregation" multiplicity: "1..*" }

association "reduces" { id: "AS-COLLECTIVE-TENSOR" from: "CL-COLLECTIVE" to: "CL-TENSOR" role: "buffer" kind: "association" multiplicity: "1" }

exchange_item "KernelLaunchRequest" { id: "EI-KERNEL-LAUNCH" mechanism: "OPERATION" elements: ["CL-KERNEL-LAUNCH"] }
exchange_item "TensorPayload" { id: "EI-TENSOR" mechanism: "FLOW" elements: ["CL-TENSOR"] }
exchange_item "CollectiveStep" { id: "EI-COLLECTIVE" mechanism: "FLOW" elements: ["CL-COLLECTIVE"] }
exchange_item "TelemetryStream" { id: "EI-TELEMETRY" mechanism: "FLOW" elements: ["CL-TELEMETRY"] }
exchange_item "ThermalAlert" { id: "EI-THERMAL-EVENT" mechanism: "EVENT" elements: ["CL-THERMAL-EVENT"] }
exchange_item "AttestationEvidence" { id: "EI-ATTESTATION" mechanism: "DATA" elements: ["CL-ATTESTATION"] }

// ---------------------------------------------------------------------------
// Requirements
// ---------------------------------------------------------------------------

requirements {
    requirement "REQ-PERF-001" {
        description: "Each accelerator shall provide at least 141 GB of HBM3e memory usable by a single job"
        priority: "Critical" category: "performance" verification_method: "test"
    }
    requirement "REQ-PERF-002" {
        description: "Each accelerator shall sustain 4.8 TB/s of HBM bandwidth on a streaming read benchmark"
        priority: "Critical" category: "performance" verification_method: "test"
    }
    requirement "REQ-PERF-003" {
        description: "An all-reduce of a 1 GB BF16 buffer across the 8 accelerators of the tray shall complete within 2.5 ms"
        priority: "Critical" category: "performance" verification_method: "test"
    }
    requirement "REQ-PERF-004" {
        description: "Host-to-device tensor transfers over the PCIe Gen5 x16 link shall sustain at least 56 GB/s"
        priority: "High" category: "performance" verification_method: "test"
    }
    requirement "REQ-THM-001" {
        description: "When the junction temperature exceeds 363 K (90 °C) the accelerator shall cap its clocks within 100 ms"
        priority: "Critical" category: "thermal" safety_level: "SIL-2" verification_method: "test"
    }
    requirement "REQ-THM-002" {
        description: "The tray shall deliver full performance with a coolant inlet temperature of 308 K (35 °C) and a flow of 1.5 L/min per module"
        priority: "High" category: "thermal" verification_method: "test"
    }
    requirement "REQ-THM-003" {
        description: "A coolant leak detected on the baseboard shall cut the 54 V bus of the affected module within 500 ms"
        priority: "Critical" category: "thermal" safety_level: "SIL-2" verification_method: "test"
    }
    requirement "REQ-PWR-001" {
        description: "Each accelerator shall not exceed a 700 W TDP and the tray shall not exceed 10.2 kW at the power shelf"
        priority: "Critical" category: "power" safety_level: "SIL-2" verification_method: "test"
    }
    requirement "REQ-PWR-002" {
        description: "A power excursion above 110 percent of TDP shall be braked to the TDP within 1 ms"
        priority: "Critical" category: "power" safety_level: "SIL-2" verification_method: "test"
    }
    requirement "REQ-RAS-001" {
        description: "HBM shall be protected by SECDED ECC; an uncorrectable error shall abort only the job owning the page and shall retire the page"
        priority: "High" category: "RAS" verification_method: "test"
    }
    requirement "REQ-RAS-002" {
        description: "The accelerator module shall demonstrate an MTBF of at least 1,000,000 h at a 308 K coolant inlet"
        priority: "High" category: "RAS" verification_method: "analysis"
    }
    requirement "REQ-SEC-001" {
        description: "At boot each accelerator shall measure its firmware, sign the measurement with a device-unique key and shall not be admitted to the pool until the orchestration agent has verified the report"
        priority: "Critical" category: "security" verification_method: "test"
    }
    requirement "REQ-SEC-002" {
        description: "Tenant partitions of one accelerator shall be isolated: no partition shall read, write or time the memory, the caches or the fabric traffic of another"
        priority: "Critical" category: "security" verification_method: "test"
    }
    requirement "REQ-SEC-003" {
        description: "Fabric traffic leaving the tray shall be authenticated and encrypted with keys rotated at least every 24 h"
        priority: "High" category: "security" verification_method: "inspection"
    }
    requirement "REQ-OBS-001" {
        description: "Each accelerator shall report temperature, power, utilization and ECC counters at 1 Hz to the management controller"
        priority: "High" category: "observability" verification_method: "test"
    }
    requirement "REQ-OBS-002" {
        description: "Thermal and power events shall reach the orchestration agent within 1 s with a nanosecond timestamp"
        priority: "Medium" category: "observability" verification_method: "demonstration"
    }
}

// ---------------------------------------------------------------------------
// Operational analysis
// ---------------------------------------------------------------------------

operational_analysis "Operational Analysis" {
    actor "ML platform engineer" {
        id: "OA-MLE"
        description: "Submits training and inference jobs, tunes collective communication"
    }
    actor "Data-center operator" {
        id: "OA-OPS"
        description: "Runs the facility: power, cooling, fleet health"
    }
    actor "Security officer" {
        id: "OA-SEC"
        description: "Owns the firmware trust policy and the tenant isolation rules"
    }
    actor "Cloud tenant" {
        id: "OA-TENANT"
        description: "Rents accelerator partitions and brings model weights and data"
    }
    entity "Data center" {
        id: "OA-DC"
        description: "The facility hosting the racks"
        activity "Provide power and cooling" {
            id: "OA-ACT-FACILITY"
            description: "54 V busbar per rack, liquid loop at 308 K inlet"
        }
        activity "Monitor the fleet" {
            id: "OA-ACT-MONITOR"
        }
    }
    entity "Cluster scheduler" {
        id: "OA-SCHED"
        description: "Places jobs on healthy, attested accelerators"
        activity "Place training jobs" {
            id: "OA-ACT-PLACE"
        }
        activity "Admit nodes to the pool" {
            id: "OA-ACT-ADMIT"
        }
    }
    entity "Accelerator tray" {
        id: "OA-TRAY"
        description: "Eight accelerators, a host and their management"
        activity "Train a model" {
            id: "OA-ACT-TRAIN"
        }
        activity "Serve inference" {
            id: "OA-ACT-SERVE"
        }
        activity "Stay within the thermal and power envelope" {
            id: "OA-ACT-ENVELOPE"
        }
        activity "Prove firmware integrity" {
            id: "OA-ACT-ATTEST"
        }
    }
    operational_capability "Train a foundation model" {
        id: "OC-TRAIN" description: "Weeks-long distributed training over thousands of accelerators"
        involves: ["OA-MLE", "OA-SCHED", "OA-TRAY", "OA-ACT-PLACE", "OA-ACT-TRAIN"]
    }
    operational_capability "Serve inference at scale" {
        id: "OC-SERVE"
        involves: ["OA-TENANT", "OA-SCHED", "OA-TRAY", "OA-ACT-SERVE"]
    }
    operational_capability "Operate safely under power and thermal limits" {
        id: "OC-SAFE"
        involves: ["OA-OPS", "OA-DC", "OA-TRAY", "OA-ACT-FACILITY", "OA-ACT-ENVELOPE", "OA-ACT-MONITOR"]
    }
    operational_capability "Trust the fleet's firmware" {
        id: "OC-TRUST"
        involves: ["OA-SEC", "OA-SCHED", "OA-TRAY", "OA-ACT-ATTEST", "OA-ACT-ADMIT"]
    }
    interaction "Job submission" { id: "OI-JOB-SUBMISSION" from: "OA-MLE" to: "OA-ACT-PLACE" }
    interaction "Job placement" { id: "OI-JOB-PLACEMENT" from: "OA-ACT-PLACE" to: "OA-ACT-TRAIN" }
    interaction "Inference request" { id: "OI-INFERENCE-REQUEST" from: "OA-TENANT" to: "OA-ACT-SERVE" }
    interaction "Power and coolant" { id: "OI-POWER-AND-COOLANT" from: "OA-ACT-FACILITY" to: "OA-ACT-ENVELOPE" }
    interaction "Fleet telemetry" { id: "OI-FLEET-TELEMETRY" from: "OA-ACT-ENVELOPE" to: "OA-ACT-MONITOR" }
    interaction "Fleet health" { id: "OI-FLEET-HEALTH" from: "OA-ACT-MONITOR" to: "OA-OPS" }
    interaction "Trust policy" { id: "OI-TRUST-POLICY" from: "OA-SEC" to: "OA-ACT-ADMIT" }
    interaction "Attestation evidence" { id: "OI-ATTESTATION-EVIDENCE" from: "OA-ACT-ATTEST" to: "OA-ACT-ADMIT" }
}

// ---------------------------------------------------------------------------
// System analysis
// ---------------------------------------------------------------------------

system_analysis "System Analysis" {
    mission "Deliver trustworthy AI compute at data-center scale" {
        id: "MIS-001"
    }

    actor "Cluster scheduler" { id: "SA-SCHED" }
    actor "Data-center operator" { id: "SA-OPS" }
    actor "Facility power and cooling" { id: "SA-FACILITY" }

    capability "Distributed training step" {
        id: "CAP-TRAIN" mission: "MIS-001" realizes: "OC-TRAIN"
        involves: ["SA-SCHED", "SF-SCHED", "SF-H2D", "SF-EXEC", "SF-REDUCE", "SF-ALLRED", "SF-SYNC", "FC-ALLREDUCE", "FC-H2D"]
    }
    capability "Partitioned inference" {
        id: "CAP-SERVE" mission: "MIS-001" realizes: "OC-SERVE"
        involves: ["SA-SCHED", "SF-ISOLATE", "SF-SCHED", "SF-EXEC"]
    }
    capability "Thermal and power protection" {
        id: "CAP-SAFE" mission: "MIS-001" realizes: "OC-SAFE"
        involves: ["SA-FACILITY", "SA-OPS", "SF-SENSE", "SF-THROTTLE", "SF-POWERCAP", "SF-CUT", "FC-THROTTLE", "FC-POWERBRAKE"]
    }
    capability "Attested boot" {
        id: "CAP-TRUST" mission: "MIS-001" realizes: "OC-TRUST"
        involves: ["SA-SCHED", "SF-ATTEST", "SF-ADMIT", "FC-BOOT"]
    }
    capability "Fleet observability" {
        id: "CAP-OBS" mission: "MIS-001" realizes: "OC-SAFE"
        involves: ["SA-OPS", "SF-TELEM", "SF-ECC"]
    }

    // Compute path
    function "Schedule kernels" {
        id: "SF-SCHED"
        description: "Turns a graph of operators into kernel launches on device streams"
        latency: 20 us
        port in "job"
        port out "launch" { data_type: "KernelLaunchRequest" type: "control" }
        port out "copy"
    }
    function "Move tensors host to device" {
        id: "SF-H2D"
        description: "DMA of weights and activations over the host link"
        latency: 18 ms
        port in "copy"
        port out "tensor" { data_type: "TensorPayload" type: "data" }
    }
    function "Execute tensor kernels" {
        id: "SF-EXEC"
        latency: 400 us
        port in "launch"
        port in "tensor"
        port out "activations"
    }
    function "Reduce gradients locally" {
        id: "SF-REDUCE"
        latency: 150 us
        port in "activations"
        port out "partial" { data_type: "CollectiveStep" type: "data" }
    }
    function "Exchange partial sums over the fabric" {
        id: "SF-ALLRED"
        description: "Ring all-reduce across the 8 devices of the tray"
        latency: 1900 us
        port in "partial"
        port out "reduced"
    }
    function "Signal step completion" {
        id: "SF-SYNC"
        latency: 10 us
        port in "reduced"
        port out "done" { type: "event" }
    }
    function "Isolate tenant partitions" {
        id: "SF-ISOLATE"
        description: "Memory, cache and fabric partitioning per tenant"
        latency: 50 ms
        port in "job"
        port out "partition"
    }

    // Protection path
    function "Sense temperature and power" {
        id: "SF-SENSE"
        is: "Periodic monitor"
        latency: 100 us
        period: 100 us
        port out "sample" { data_type: "TelemetrySample" type: "data" }
    }
    function "Throttle clocks on thermal event" {
        id: "SF-THROTTLE"
        latency: 5 ms
        safety_level: "SIL-2"
        port in "sample"
        port out "clock cap" { type: "control" }
        port out "event" { data_type: "ThermalAlert" type: "event" }
    }
    function "Brake a power excursion" {
        id: "SF-POWERCAP"
        latency: 500 us
        safety_level: "SIL-2"
        port in "sample"
        port out "power cap" { type: "control" }
    }
    function "Cut module power on leak" {
        id: "SF-CUT"
        latency: 200 ms
        safety_level: "SIL-2"
        port in "leak"
        port out "bus off" { type: "control" }
    }
    function "Correct and contain memory errors" {
        id: "SF-ECC"
        latency: 1 ms
        port out "ecc counters"
    }

    // Trust and observability path
    function "Attest firmware at boot" {
        id: "SF-ATTEST"
        description: "Measured boot: SHA-384 of every firmware stage, signed with the device key"
        latency: 2 s
        port out "report" { data_type: "AttestationEvidence" type: "data" }
    }
    function "Admit the device to the pool" {
        id: "SF-ADMIT"
        latency: 500 ms
        port in "report"
        port out "admitted" { type: "event" }
    }
    function "Report telemetry" {
        id: "SF-TELEM"
        is: "Periodic monitor"
        latency: 20 ms
        period: 1 s
        port in "sample"
        port in "event"
        port in "ecc counters"
        port out "telemetry" { data_type: "TelemetryStream" type: "data" }
    }

    functional_exchange "kernel launches" { from: "SF-SCHED.launch" to: "SF-EXEC.launch" exchange_items: ["EI-KERNEL-LAUNCH"] }
    functional_exchange "copy requests" { from: "SF-SCHED.copy" to: "SF-H2D.copy" }
    functional_exchange "input tensors" { from: "SF-H2D.tensor" to: "SF-EXEC.tensor" exchange_items: ["EI-TENSOR"] bandwidth: 448 Gbps }
    functional_exchange "activations" { from: "SF-EXEC.activations" to: "SF-REDUCE.activations" }
    functional_exchange "partial sums" { from: "SF-REDUCE.partial" to: "SF-ALLRED.partial" exchange_items: ["EI-COLLECTIVE"] bandwidth: 7200 Gbps }
    functional_exchange "reduced gradients" { from: "SF-ALLRED.reduced" to: "SF-SYNC.reduced" }
    functional_exchange "sensor samples" { from: "SF-SENSE.sample" to: "SF-THROTTLE.sample" }
    functional_exchange "power samples" { from: "SF-SENSE.sample" to: "SF-POWERCAP.sample" }
    functional_exchange "telemetry samples" { from: "SF-SENSE.sample" to: "SF-TELEM.sample" period: 1 s }
    functional_exchange "thermal events" { from: "SF-THROTTLE.event" to: "SF-TELEM.event" exchange_items: ["EI-THERMAL-EVENT"] }
    functional_exchange "ecc counters" { from: "SF-ECC.ecc counters" to: "SF-TELEM.ecc counters" }
    functional_exchange "attestation report" { from: "SF-ATTEST.report" to: "SF-ADMIT.report" exchange_items: ["EI-ATTESTATION"] size: 4 kB }

    functional_chain "All-reduce step" {
        id: "FC-ALLREDUCE"
        description: "One gradient synchronization step: local reduce, ring exchange, completion"
        involves: ["SF-SCHED", "SF-REDUCE", "SF-ALLRED", "SF-SYNC"]
        latency_budget: 2500 us
        capability: "CAP-TRAIN"
    }
    functional_chain "Host to device transfer" {
        id: "FC-H2D"
        involves: ["SF-SCHED", "SF-H2D"]
        latency_budget: 20 ms
        capability: "CAP-TRAIN"
    }
    functional_chain "Thermal throttling" {
        id: "FC-THROTTLE"
        description: "From the sensor sample crossing the threshold to the reported, capped device"
        involves: ["SF-SENSE", "SF-THROTTLE", "SF-TELEM"]
        latency_budget: 100 ms
        capability: "CAP-SAFE"
    }
    functional_chain "Power brake" {
        id: "FC-POWERBRAKE"
        involves: ["SF-SENSE", "SF-POWERCAP"]
        latency_budget: 1 ms
        capability: "CAP-SAFE"
    }
    functional_chain "Attested boot" {
        id: "FC-BOOT"
        involves: ["SF-ATTEST", "SF-ADMIT"]
        latency_budget: 30 s
        capability: "CAP-TRUST"
    }
}

// ---------------------------------------------------------------------------
// Logical architecture
// ---------------------------------------------------------------------------

logical_architecture "Logical Architecture" {
    component "AI accelerator" {
        id: "LC-ACC"
        description: "One accelerator device: compute die, HBM3e, fabric and host endpoints, power and thermal management, security processor"
        multiplicity: 8
        power: 700 W
        ram: 141 GB
        memory_bandwidth: 38400 Gbps
        mtbf: 1000000 h
        port in "launch" { interface: "Kernel launch" protocol: "PCIe Gen5 x16" }
        port in "tensors" { interface: "Tensor DMA" protocol: "PCIe Gen5 x16" }
        port inout "fabric" { interface: "Collective fabric" }
        port out "telemetry" { interface: "Device telemetry" protocol: "PLDM over SMBus" }
        port in "controls" { interface: "Device management" protocol: "PLDM over SMBus" }
        port out "attestation" { interface: "Attestation" protocol: "SPDM" }

        component "Compute die" {
            id: "LC-CDIE"
            description: "Streaming multiprocessor clusters sharing a 50 MB L2 cache"
            port in "kernels"
            port in "operands"
            port out "activations"
            component "Tensor cores" {
                id: "LC-TC"
                description: "Mixed-precision matrix units: FP8, BF16, FP16, TF32"
                multiplicity: 528
                function "Run matrix kernels" { id: "LF-MATMUL" realizes: "SF-EXEC" }
            }
            component "SM clusters" {
                id: "LC-SMC"
                description: "Streaming multiprocessors with their schedulers and register files"
                multiplicity: 132
                function "Dispatch warps" { id: "LF-DISPATCH" realizes: "SF-EXEC" }
                function "Reduce gradients in L2" { id: "LF-REDUCE" realizes: "SF-REDUCE" }
            }
            function "Enforce partition boundaries" { id: "LF-PARTITION" description: "Address-space and cache-way partitioning per tenant" realizes: "SF-ISOLATE" }
        }

        component "Memory subsystem" {
            id: "LC-MEM"
            description: "Memory controllers and six HBM3e stacks on a 6144-bit interface"
            ram: 141 GB
            port in "write"
            port out "read"
            component "HBM3e stacks" {
                id: "LC-HBM"
                multiplicity: 6
                ram: 24 GB
                function "Store tensors" { id: "LF-STORE" realizes: "SF-H2D" }
            }
            function "Correct errors with SECDED ECC" { id: "LF-ECC" realizes: "SF-ECC" }
        }

        component "Fabric endpoint" {
            id: "LC-FAB"
            description: "18 fabric ports, 900 GB/s aggregate per device"
            port in "partials"
            port inout "links"
            function "Run ring collectives" { id: "LF-COLLECTIVE" realizes: "SF-ALLRED" }
            function "Signal completion" { id: "LF-SYNC" realizes: "SF-SYNC" }
        }

        component "Host interface" {
            id: "LC-HIF"
            description: "PCIe Gen5 x16 endpoint with CXL.mem for coherent host memory"
            port in "pcie"
            port out "doorbells"
            port out "dma"
            function "Receive kernel launches" { id: "LF-LAUNCH-RX" realizes: "SF-SCHED" }
            function "DMA tensors into HBM" { id: "LF-DMA" realizes: "SF-H2D" }
        }

        component "Power delivery" {
            id: "LC-PDN"
            description: "54 V to 0.8 V conversion with a hardware power brake"
            safety_level: "SIL-2"
            power: 700 W
            voltage: 54 V
            port in "cap"
            function "Brake power within TDP" { id: "LF-POWERBRAKE" realizes: "SF-POWERCAP" }
        }

        component "Thermal manager" {
            id: "LC-THM"
            description: "Junction and HBM temperature sensing, clock capping"
            safety_level: "SIL-2"
            port out "samples"
            port out "clock cap"
            function "Sample sensors at 10 kHz" { id: "LF-SENSE" realizes: "SF-SENSE" }
            function "Cap clocks on threshold" { id: "LF-THROTTLE" realizes: "SF-THROTTLE" }
        }

        component "Security processor" {
            id: "LC-SEC"
            description: "Root of trust: measured boot, device identity key, SPDM responder"
            flash: 64 MB
            port out "report"
            function "Measure and sign firmware" { id: "LF-ATTEST" realizes: "SF-ATTEST" }
            function "Partition the fabric per tenant" { id: "LF-FABRIC-ISOLATE" realizes: "SF-ISOLATE" }
        }

        function "Publish device telemetry" { id: "LF-TELEM" realizes: "SF-TELEM" }
    }

    component "Fabric switch" {
        id: "LC-FSW"
        description: "Tray-level switch joining the 8 accelerators and the scale-out ports; link encryption on the scale-out side"
        multiplicity: 4
        port inout "device ports"
        port inout "scale-out"
        function "Switch collective traffic" { id: "LF-SWITCH" realizes: "SF-ALLRED" }
        function "Encrypt scale-out links" { id: "LF-LINK-CRYPTO" description: "AES-GCM per link, keys rotated every 24 h" realizes: "SF-ALLRED" }
    }

    component "Management controller" {
        id: "LC-BMC"
        description: "Baseboard management controller: telemetry aggregation, thermal and power policy, Redfish to the operator"
        safety_level: "SIL-2"
        ram: 2 GB
        port in "device telemetry"
        port out "device controls"
        port in "attestation"
        port out "fleet telemetry"
        port out "coolant setpoint"
        port in "leak"
        port out "bus control"
        component "Attestation agent" {
            id: "LC-ATT"
            description: "SPDM requester collecting the reports of the 8 devices"
            port out "evidence"
            function "Collect attestation reports" { id: "LF-COLLECT-ATTEST" realizes: "SF-ATTEST" }
        }
        function "Aggregate telemetry at 1 Hz" { id: "LF-AGGREGATE" realizes: "SF-TELEM" }
        function "Apply thermal policy" { id: "LF-POLICY" realizes: "SF-THROTTLE" }
        function "Cut the module bus on leak" { id: "LF-CUT" realizes: "SF-CUT" }
    }

    component "Coolant loop controller" {
        id: "LC-CLC"
        description: "Pump speed and inlet temperature control of the tray's liquid loop"
        safety_level: "SIL-2"
        port in "setpoint"
        port out "leak"
        function "Hold the inlet at 308 K" { id: "LF-COOL" realizes: "SF-SENSE" }
        function "Detect leaks" { id: "LF-LEAK" realizes: "SF-CUT" }
    }

    component "Host runtime stack" {
        id: "LC-HOST"
        description: "Driver, runtime and collective library on the host CPU"
        port in "jobs"
        port out "launch"
        port out "tensors"
        port in "completion"
        component "Device driver" {
            id: "LC-DRV"
            port out "doorbells"
            function "Map device memory and rings" { id: "LF-MAP" realizes: "SF-SCHED" }
        }
        component "Runtime and collective library" {
            id: "LC-RT"
            port in "graph"
            function "Schedule kernels on streams" { id: "LF-SCHED" realizes: "SF-SCHED" }
            function "Stage tensors for DMA" { id: "LF-STAGE" realizes: "SF-H2D" }
            function "Plan ring all-reduce" { id: "LF-RING" realizes: "SF-ALLRED" }
        }
    }

    component "Orchestration agent" {
        id: "LC-ORCH"
        description: "Node agent of the cluster scheduler: admits attested devices, exports telemetry, receives jobs"
        port in "evidence"
        port in "fleet telemetry"
        port out "jobs"
        port out "pool status"
        function "Verify attestation and admit" { id: "LF-ADMIT" realizes: "SF-ADMIT" }
        function "Export telemetry to the fleet" { id: "LF-EXPORT" realizes: "SF-TELEM" }
    }

    interface "Kernel launch" {
        id: "IF-LAUNCH"
        description: "Doorbell writes and launch descriptors over MMIO"
        from: "LC-HOST"
        to: "LC-ACC"
        exchange_items: ["EI-KERNEL-LAUNCH"]
    }
    interface "Tensor DMA" {
        id: "IF-DMA"
        description: "Bulk transfers at 56 GB/s sustained"
        from: "LC-HOST"
        to: "LC-ACC"
        exchange_items: ["EI-TENSOR"]
    }
    interface "Collective fabric" {
        id: "IF-FABRIC"
        description: "Load/store and collective primitives across devices at 900 GB/s per device"
        from: "LC-ACC"
        to: "LC-FSW"
        exchange_items: ["EI-COLLECTIVE"]
    }
    interface "Device telemetry" {
        id: "IF-TELEMETRY"
        description: "PLDM sensor readings at 1 Hz"
        from: "LC-ACC"
        to: "LC-BMC"
        exchange_items: ["EI-TELEMETRY", "EI-THERMAL-EVENT"]
    }
    interface "Attestation" {
        id: "IF-ATTESTATION"
        description: "SPDM GET_MEASUREMENTS with a fresh nonce"
        from: "LC-ACC"
        to: "LC-BMC"
        exchange_items: ["EI-ATTESTATION"]
    }

    component_exchange "Kernel launches" { id: "CE-KERNEL-LAUNCHES" from_port: "LC-HOST.launch" to_port: "LC-ACC.launch" exchange_items: ["EI-KERNEL-LAUNCH"] protocol: "PCIe Gen5 x16" }
    component_exchange "Tensor transfers" { id: "CE-TENSOR-TRANSFERS" from_port: "LC-HOST.tensors" to_port: "LC-ACC.tensors" exchange_items: ["EI-TENSOR"] bandwidth: 448 Gbps }
    component_exchange "Step completion" { id: "CE-STEP-COMPLETION" from_port: "LC-ACC.telemetry" to_port: "LC-HOST.completion" }
    component_exchange "Collective traffic" { id: "CE-COLLECTIVE-TRAFFIC" from_port: "LC-ACC.fabric" to_port: "LC-FSW.device ports" exchange_items: ["EI-COLLECTIVE"] bandwidth: 7200 Gbps }
    component_exchange "Telemetry stream" { id: "CE-TELEMETRY-STREAM" from_port: "LC-ACC.telemetry" to_port: "LC-BMC.device telemetry" exchange_items: ["EI-TELEMETRY", "EI-THERMAL-EVENT"] period: 1 s }
    component_exchange "Device controls" { id: "CE-DEVICE-CONTROLS" from_port: "LC-BMC.device controls" to_port: "LC-ACC.controls" }
    component_exchange "Attestation reports" { id: "CE-ATTESTATION-REPORTS" from_port: "LC-ACC.attestation" to_port: "LC-BMC.attestation" exchange_items: ["EI-ATTESTATION"] }
    component_exchange "Attestation evidence" { id: "CE-ATTESTATION-EVIDENCE" from_port: "LC-BMC.fleet telemetry" to_port: "LC-ORCH.evidence" exchange_items: ["EI-ATTESTATION"] }
    component_exchange "Fleet telemetry" { id: "CE-FLEET-TELEMETRY" from_port: "LC-BMC.fleet telemetry" to_port: "LC-ORCH.fleet telemetry" exchange_items: ["EI-TELEMETRY"] protocol: "Redfish" }
    component_exchange "Coolant setpoint" { id: "CE-COOLANT-SETPOINT" from_port: "LC-BMC.coolant setpoint" to_port: "LC-CLC.setpoint" }
    component_exchange "Leak alarm" { id: "CE-LEAK-ALARM" from_port: "LC-CLC.leak" to_port: "LC-BMC.leak" }
    component_exchange "Job dispatch" { id: "CE-JOB-DISPATCH" from_port: "LC-ORCH.jobs" to_port: "LC-HOST.jobs" }

    // Inside one accelerator
    component_exchange "Doorbells to the die" { id: "CE-INT-LAUNCH" from_port: "LC-HIF.doorbells" to_port: "LC-CDIE.kernels" exchange_items: ["EI-KERNEL-LAUNCH"] }
    component_exchange "DMA into HBM" { id: "CE-INT-DMA" from_port: "LC-HIF.dma" to_port: "LC-MEM.write" exchange_items: ["EI-TENSOR"] bandwidth: 448 Gbps }
    component_exchange "Operand reads" { id: "CE-INT-READ" from_port: "LC-MEM.read" to_port: "LC-CDIE.operands" bandwidth: 38400 Gbps }
    component_exchange "Partial sums to the fabric" { id: "CE-INT-PARTIALS" from_port: "LC-CDIE.activations" to_port: "LC-FAB.partials" exchange_items: ["EI-COLLECTIVE"] }
    component_exchange "Clock and power cap" { id: "CE-INT-CAP" from_port: "LC-THM.clock cap" to_port: "LC-PDN.cap" }

    capability_realization "Distributed training step" {
        id: "CR-TRAIN" realizes: "CAP-TRAIN"
        involves: ["LC-HOST", "LC-ACC", "LC-FSW"]
    }
    capability_realization "Thermal and power protection" {
        id: "CR-SAFE" realizes: "CAP-SAFE"
        involves: ["LC-THM", "LC-PDN", "LC-BMC", "LC-CLC"]
    }
    capability_realization "Attested boot" {
        id: "CR-TRUST" realizes: "CAP-TRUST"
        involves: ["LC-SEC", "LC-ATT", "LC-ORCH"]
    }
}

// ---------------------------------------------------------------------------
// Physical architecture
// ---------------------------------------------------------------------------

physical_architecture "Physical Architecture" {
    node "Host CPU server" {
        id: "PN-HOST"
        description: "Dual-socket head node of the tray"
        processor: "2 x 64-core server CPU, 5 GHz boost"
        ram: 2048 GB
        storage: 30000 GB
        power: 1500 W
        port "pcie root" { connector: "PCIe Gen5 x16 CEM" }
        port "mgmt" { connector: "RJ45" }
        deploys "LC-HOST"
        deploys "LC-DRV"
        deploys "LC-RT"
        deploys "LC-ORCH"
        behavior_component "Driver and runtime" {
            id: "BC-RUNTIME"
            allocated_component: "LC-HOST"
            function "Enqueue kernels" { id: "PF-ENQUEUE" realizes: "LF-SCHED" }
            function "Pin and stage host buffers" { id: "PF-STAGE" realizes: "LF-STAGE" }
        }
        behavior_component "Node agent" {
            id: "BC-AGENT"
            allocated_component: "LC-ORCH"
            function "Verify SPDM evidence" { id: "PF-VERIFY" realizes: "LF-ADMIT" }
        }
    }

    node "Accelerator module (OAM)" {
        id: "PN-OAM"
        description: "OCP Accelerator Module: die, HBM3e, voltage regulators, cold plate"
        multiplicity: 8
        power: 700 W
        voltage: 54 V
        ram: 141 GB
        mass: 2 kg
        port "pcie" { connector: "OAM mezzanine, PCIe Gen5 x16" }
        port "fabric" { connector: "OAM mezzanine, 7 x 4 lanes" }
        port "smbus" { connector: "OAM mezzanine, SMBus" }
        port "coolant" { connector: "Quick disconnect, 1.5 L/min" }
        deploys "LC-ACC"
        deploys "LC-CDIE"
        deploys "LC-TC"
        deploys "LC-SMC"
        deploys "LC-MEM"
        deploys "LC-HBM"
        deploys "LC-FAB"
        deploys "LC-HIF"
        deploys "LC-PDN"
        deploys "LC-THM"
        deploys "LC-SEC"
        behavior_component "Device firmware" {
            id: "BC-DEVFW"
            allocated_component: "LC-SEC"
            function "Measured boot" { id: "PF-MBOOT" realizes: "LF-ATTEST" }
            function "SPDM responder" { id: "PF-SPDM" realizes: "LF-ATTEST" }
        }
        behavior_component "Thermal and power firmware" {
            id: "BC-THMFW"
            allocated_component: "LC-THM"
            function "Sensor sampling loop" { id: "PF-SAMPLE" realizes: "LF-SENSE" }
            function "Clock capping" { id: "PF-CAP" realizes: "LF-THROTTLE" }
        }
        hardware_component "Compute die" { id: "HW-DIE" power: 550 W }
        hardware_component "HBM3e stacks" { id: "HW-HBM" ram: 144 GB power: 90 W }
        hardware_component "Voltage regulators" { id: "HW-VRM" voltage: 54 V power: 60 W }
    }

    node "Universal baseboard" {
        id: "PN-BASE"
        description: "Carrier of the 8 OAMs, the fabric switches, the PCIe retimers and the leak sensors"
        power: 400 W
        port "host uplink" { connector: "MCIO x16" }
        port "oam slots" { connector: "8 x OAM mezzanine" }
        port "scale-out" { connector: "18 x OSFP" }
        port "power in" { connector: "54 V busbar clip" }
        port "smbus" { connector: "SMBus header" }
        deploys "LC-FSW"
        behavior_component "Switch firmware" {
            id: "BC-SWFW"
            allocated_component: "LC-FSW"
            function "Program routing tables" { id: "PF-ROUTE" realizes: "LF-SWITCH" }
            function "Rotate link keys" { id: "PF-KEYS" realizes: "LF-LINK-CRYPTO" }
        }
        hardware_component "Fabric switch ASICs" { id: "HW-FSW" power: 300 W }
        hardware_component "PCIe retimers" { id: "HW-RETIMER" power: 40 W }
        hardware_component "Leak sensor rope" { id: "HW-LEAK" }
    }

    node "Tray management controller" {
        id: "PN-BMC"
        description: "BMC SoC with its own flash and the out-of-band Ethernet"
        processor: "Quad-core management SoC, 1.2 GHz"
        ram: 2 GB
        flash: 512 MB
        power: 15 W
        port "mgmt eth" { connector: "RJ45, 1 GbE" }
        port "smbus" { connector: "SMBus header" }
        port "pmbus" { connector: "PMBus header" }
        deploys "LC-BMC"
        deploys "LC-ATT"
        behavior_component "BMC firmware" {
            id: "BC-BMCFW"
            allocated_component: "LC-BMC"
            function "PLDM telemetry poller" { id: "PF-POLL" realizes: "LF-AGGREGATE" }
            function "Thermal policy engine" { id: "PF-POLICY" realizes: "LF-POLICY" }
            function "SPDM requester" { id: "PF-SPDM-REQ" realizes: "LF-COLLECT-ATTEST" }
        }
    }

    node "Rack power shelf" {
        id: "PN-PSHELF"
        description: "Rectifiers feeding the 54 V rack busbar; 33 kW per shelf"
        power: 33 kW
        voltage: 54 V
        redundancy: "N+1 rectifiers"
        port "busbar" { connector: "54 V busbar" }
        port "pmbus" { connector: "PMBus" }
    }

    node "Cold plate and coolant distribution unit" {
        id: "PN-CDU"
        description: "In-rack CDU: pumps, heat exchanger, 308 K supply to the cold plates"
        power: 2 kW
        coolant_inlet: 308 K
        port "supply" { connector: "Manifold, 1.5 L/min per module" }
        port "control" { connector: "RJ45, Modbus TCP" }
        deploys "LC-CLC"
        behavior_component "CDU controller firmware" {
            id: "BC-CDUFW"
            allocated_component: "LC-CLC"
            function "Pump and valve control" { id: "PF-PUMP" realizes: "LF-COOL" }
            function "Leak rope monitoring" { id: "PF-LEAK" realizes: "LF-LEAK" }
        }
    }

    node "Rack fabric switch" {
        id: "PN-RACKSW"
        description: "Scale-out switch joining the trays of the rack"
        power: 1200 W
        port "downlinks" { connector: "64 x OSFP" }
    }

    physical_link "Host PCIe link" {
        id: "PL-PCIE-HOST" from: "PN-HOST.pcie root" to: "PN-BASE.host uplink"
        protocol: "PCIe Gen5 x16" bandwidth: 512 Gbps latency: 1 us
    }
    physical_link "OAM PCIe lanes" {
        id: "PL-PCIE-OAM" from: "PN-BASE.oam slots" to: "PN-OAM.pcie"
        protocol: "PCIe Gen5 x16" bandwidth: 512 Gbps latency: 500 ns
    }
    physical_link "Intra-tray fabric" {
        id: "PL-FABRIC" is: "Fabric link" from: "PN-OAM.fabric" to: "PN-BASE.oam slots"
        protocol: "GPU-to-GPU fabric, 18 ports x 50 GB/s"
    }
    physical_link "Scale-out fabric cables" {
        id: "PL-SCALEOUT" is: "Fabric link" from: "PN-BASE.scale-out" to: "PN-RACKSW.downlinks"
        protocol: "GPU-to-GPU fabric over OSFP, 800 Gbps per port, AES-GCM" bandwidth: 14400 Gbps
    }
    physical_link "Device management bus" {
        id: "PL-SMBUS" from: "PN-BMC.smbus" to: "PN-OAM.smbus"
        protocol: "PLDM and SPDM over MCTP over SMBus" bandwidth: 1 Mbps
    }
    physical_link "Baseboard management bus" {
        id: "PL-SMBUS-BASE" from: "PN-BMC.smbus" to: "PN-BASE.smbus"
        protocol: "MCTP over SMBus" bandwidth: 1 Mbps
    }
    physical_link "Management Ethernet" {
        id: "PL-MGMT" from: "PN-BMC.mgmt eth" to: "PN-HOST.mgmt"
        protocol: "Redfish over HTTPS, 1 GbE" bandwidth: 1 Gbps
    }
    physical_link "CDU control link" {
        id: "PL-CDU" from: "PN-BMC.mgmt eth" to: "PN-CDU.control"
        protocol: "Modbus TCP over 1 GbE" bandwidth: 1 Gbps
    }
    physical_link "Power shelf bus" {
        id: "PL-PMBUS" from: "PN-BMC.pmbus" to: "PN-PSHELF.pmbus"
        protocol: "PMBus" bandwidth: 400 kbps
    }
    physical_link "54 V busbar" {
        id: "PL-POWER" from: "PN-PSHELF.busbar" to: "PN-BASE.power in"
        protocol: "54 V DC busbar, 10.2 kW per tray"
    }
    physical_link "Coolant loop" {
        id: "PL-COOLANT" from: "PN-CDU.supply" to: "PN-OAM.coolant"
        protocol: "Water-glycol loop, 308 K supply"
    }

    physical_exchange "Kernel launch doorbells" {
        id: "PX-DOORBELL" from: "PN-HOST" to: "PN-OAM" via: "PL-PCIE-HOST"
        message_type: "MMIO write, 64 B descriptor" frequency: "100 kHz" size: 64 B
    }
    physical_exchange "Tensor DMA" {
        id: "PX-DMA" from: "PN-HOST" to: "PN-OAM" via: "PL-PCIE-HOST"
        message_type: "PCIe memory write TLPs, 1 GB per transfer" frequency: "50 Hz" bandwidth: 448 Gbps
    }
    physical_exchange "Telemetry samples" {
        id: "PX-TELEMETRY" from: "PN-OAM" to: "PN-BMC" via: "PL-SMBUS"
        message_type: "PLDM GetSensorReading, 256 B" frequency: "1 Hz" size: 256 B
    }
    physical_exchange "Attestation report" {
        id: "PX-ATTEST" from: "PN-OAM" to: "PN-BMC" via: "PL-SMBUS"
        message_type: "SPDM MEASUREMENTS response, 4 kB, at boot then every 100 s" frequency: "0.01 Hz" size: 4 kB
    }
    physical_exchange "Fleet telemetry" {
        id: "PX-FLEET" from: "PN-BMC" to: "PN-HOST" via: "PL-MGMT"
        message_type: "Redfish telemetry report, JSON" frequency: "1 Hz" size: 16 kB
    }
    physical_exchange "Coolant setpoint" {
        id: "PX-COOLANT" from: "PN-BMC" to: "PN-CDU" via: "PL-CDU"
        message_type: "Modbus holding register write" frequency: "1 Hz"
    }

    physical_path "Host to device launch path" {
        id: "PP-LAUNCH"
        involves: ["Host PCIe link", "OAM PCIe lanes"]
    }
    physical_path "Cross-tray collective path" {
        id: "PP-COLLECTIVE"
        involves: ["Intra-tray fabric", "Scale-out fabric cables"]
    }
}

// ---------------------------------------------------------------------------
// EPBS: configuration items
// ---------------------------------------------------------------------------

epbs "Product Breakdown" {
    system "AI accelerator compute tray" {
        id: "CI-TRAY"
        kind: "SystemCI"
        subsystem "Accelerator modules" {
            id: "CI-ACCEL"
            kind: "HWCI"
            realizes: ["PN-OAM"]
            item "OAM accelerator module" {
                id: "CI-OAM"
                part_number: "ACC-OAM-141-700"
                supplier: "In-house"
                power: 700 W
                ram: 141 GB
                realizes: ["PN-OAM"]
            }
            item "HBM3e stack" {
                id: "CI-HBM"
                part_number: "HBM3E-24G-8H"
                supplier: "Memory vendor"
                ram: 24 GB
            }
        }
        subsystem "Baseboard" {
            id: "CI-BASEBOARD"
            kind: "HWCI"
            realizes: ["PN-BASE"]
            item "Universal baseboard" { id: "CI-UBB" part_number: "UBB-8OAM-G5" supplier: "In-house" realizes: ["PN-BASE"] }
            item "Fabric switch ASIC" { id: "CI-FSW" part_number: "FSW-18P-900" supplier: "In-house" power: 75 W }
        }
        subsystem "Host and management" {
            id: "CI-HOSTMGMT"
            kind: "HWCI"
            realizes: ["PN-HOST", "PN-BMC"]
            item "Head node server" { id: "CI-SERVER" part_number: "SRV-2S-G5" supplier: "Server ODM" realizes: ["PN-HOST"] }
            item "Tray BMC" { id: "CI-BMC" part_number: "BMC-SOC-4C" supplier: "BMC vendor" realizes: ["PN-BMC"] }
        }
        subsystem "Power and cooling" {
            id: "CI-FACILITY"
            kind: "HWCI"
            realizes: ["PN-PSHELF", "PN-CDU"]
            item "Rack power shelf" { id: "CI-PSHELF" part_number: "PS-54V-33K" supplier: "Power vendor" power: 33 kW realizes: ["PN-PSHELF"] }
            item "Coolant distribution unit" { id: "CI-CDU" part_number: "CDU-RACK-80K" supplier: "Cooling vendor" realizes: ["PN-CDU"] }
        }
        subsystem "Firmware and software" {
            id: "CI-SOFTWARE"
            kind: "CSCI"
            realizes: ["BC-DEVFW", "BC-THMFW", "BC-BMCFW", "BC-SWFW", "BC-RUNTIME", "BC-AGENT", "BC-CDUFW"]
            item "Device firmware image" { id: "CI-DEVFW" part_number: "FW-DEV-4.3" supplier: "In-house" realizes: ["BC-DEVFW", "BC-THMFW"] }
            item "BMC firmware image" { id: "CI-BMCFW" part_number: "FW-BMC-4.3" supplier: "In-house" realizes: ["BC-BMCFW"] }
            item "Driver and runtime package" { id: "CI-RUNTIME" part_number: "SW-RT-4.3" supplier: "In-house" realizes: ["BC-RUNTIME", "BC-AGENT"] }
        }
    }
}

// ---------------------------------------------------------------------------
// Safety: HARA and FMEA
// ---------------------------------------------------------------------------

safety_analysis {
    hazard "Overtemperature of an accelerator" {
        id: "HZ-001" description: "Junction above 378 K with the clocks uncapped: thermal runaway, damaged die, risk of fire in the rack"
        severity: "S3" exposure: "E4" controllability: "C2" asil: "ASIL-C" condition: "Hazardous" dal: "DAL-B" mitigated_by: ["REQ-THM-001", "REQ-OBS-001"]
    }
    hazard "Power excursion beyond the shelf rating" {
        id: "HZ-002" description: "Eight devices peaking together trip the rack breaker: loss of the whole rack, stress on the busbar"
        severity: "S2" exposure: "E3" controllability: "C2" asil: "ASIL-A" condition: "Major" dal: "DAL-C" mitigated_by: ["REQ-PWR-001", "REQ-PWR-002"]
    }
    hazard "Coolant leak on an energized module" {
        id: "HZ-003" description: "Water-glycol on the 54 V plane: short circuit, electrolysis, fire"
        severity: "S3" exposure: "E2" controllability: "C3" asil: "ASIL-B" condition: "Major" dal: "DAL-C" mitigated_by: ["REQ-THM-003"]
    }

    fmea "HBM uncorrectable error" {
        id: "FM-001" failure_mode: "Double-bit error in an HBM row" effect: "Job owning the page aborts; page retired" cause: "Cell wear, row hammer, cosmic ray"
        severity: "6" occurrence: "3" detection: "2" rpn: 36 mitigation: "SECDED ECC, page retirement, row remapping (REQ-RAS-001)"
    }
    fmea "Cold plate pump failure" {
        id: "FM-002" failure_mode: "Loss of coolant flow to the tray" effect: "All eight devices throttle, then enter Fault" cause: "Pump bearing, controller fault"
        severity: "8" occurrence: "2" detection: "2" rpn: 32 mitigation: "Redundant pumps, flow sensor, throttle within 100 ms (REQ-THM-001)"
    }
    fmea "Fabric link CRC errors" {
        id: "FM-003" failure_mode: "Bit errors on a scale-out cable" effect: "Collective retries; step time grows" cause: "Connector contamination, cable bend radius"
        severity: "4" occurrence: "4" detection: "1" rpn: 16 mitigation: "Link-level retry, FEC, telemetry of CRC counters"
    }
    fmea "Voltage regulator phase failure" {
        id: "FM-004" failure_mode: "One VRM phase opens" effect: "Reduced power capability; power brake engages" cause: "MOSFET failure, inductor saturation"
        severity: "6" occurrence: "2" detection: "2" rpn: 24 mitigation: "N+2 phases, per-phase current telemetry (REQ-PWR-002)"
    }
}

// ---------------------------------------------------------------------------
// Cybersecurity
// ---------------------------------------------------------------------------

security_scales {
    id: "SEC-SCALES"
    confidentiality: ["0 public", "1 internal", "2 tenant confidential", "3 secret"]
    integrity: ["0 none", "1 detected", "2 corrected", "3 proven"]
    availability: ["0 best effort", "1 hours", "2 minutes", "3 seconds"]
    traceability: ["0 none", "1 logged", "2 signed", "3 attested"]
    threat_kinds: ["Tampering", "Information disclosure", "Eavesdropping", "Denial of service"]
}

security_analysis {
    level: "la"
    id: "SEC-LA"
    asset "Model weights" {
        id: "AS-WEIGHTS"
        kind: "information"
        description: "Tenant model parameters resident in HBM and crossing the fabric"
        members: ["CL-TENSOR", "EI-TENSOR", "EI-COLLECTIVE"]
        security_confidentiality: "3 secret"
        security_integrity: "2 corrected"
    }
    asset "Tenant data" {
        id: "AS-TENANT-DATA"
        kind: "information"
        members: ["CL-TENSOR", "CL-KERNEL-LAUNCH", "EI-KERNEL-LAUNCH"]
        security_confidentiality: "2 tenant confidential"
        security_integrity: "2 corrected"
    }
    asset "Device firmware" {
        id: "AS-FIRMWARE"
        kind: "functional"
        members: ["SF-ATTEST", "SF-THROTTLE", "SF-POWERCAP", "CL-ATTESTATION"]
        security_integrity: "3 proven"
        security_traceability: "3 attested"
        security_availability: "2 minutes"
    }
    asset "Fleet telemetry" {
        id: "AS-TELEMETRY"
        kind: "information"
        members: ["CL-TELEMETRY", "EI-TELEMETRY", "SF-TELEM"]
        security_integrity: "1 detected"
        security_availability: "2 minutes"
    }
    threat "Firmware tampering" {
        id: "TH-FIRMWARE"
        kind: "Tampering"
        level: 3
        rationale: "A modified thermal or power firmware disables the protections of HZ-001 and HZ-002; a modified boot stage hides a persistent implant"
        applies_to: ["AS-FIRMWARE"]
        involves: ["LC-SEC", "LC-THM", "LC-PDN", "LC-BMC"]
        security_integrity: "3 proven"
        security_traceability: "3 attested"
    }
    threat "Cross-tenant side channel" {
        id: "TH-SIDE-CHANNEL"
        kind: "Information disclosure"
        level: 2
        rationale: "Timing of shared L2 ways, HBM channels or fabric ports leaks weights or activations of a co-resident tenant"
        applies_to: ["AS-WEIGHTS", "AS-TENANT-DATA"]
        involves: ["LC-CDIE", "LC-MEM", "LC-FAB"]
        security_confidentiality: "3 secret"
    }
    threat "Fabric eavesdropping" {
        id: "TH-FABRIC-TAP"
        kind: "Eavesdropping"
        level: 2
        rationale: "A tapped scale-out cable exposes gradients and weights in clear"
        applies_to: ["AS-WEIGHTS"]
        involves: ["LC-FSW"]
        security_confidentiality: "3 secret"
    }
    threat "Thermal denial of service" {
        id: "TH-THERMAL-DOS"
        kind: "Denial of service"
        level: 2
        rationale: "A tenant workload crafted to peak power on all devices forces throttling of its neighbors or a breaker trip"
        applies_to: ["AS-TELEMETRY"]
        involves: ["LC-PDN", "LC-BMC"]
        security_availability: "2 minutes"
    }
}

// ---------------------------------------------------------------------------
// Modes
// ---------------------------------------------------------------------------

state_machine "Accelerator power and thermal modes" {
    initial: "Off"
    mode Off { description: "54 V bus off, auxiliary rail only" }
    mode Boot { description: "Measured boot, SPDM attestation in progress" }
    mode Idle { description: "Admitted to the pool, clocks at base, 70 W" }
    mode Compute {
        description: "Running kernels at boost clocks"
        initial: "Dedicated"
        mode Dedicated { description: "The whole device for one job" }
        mode Partitioned { description: "Up to 7 isolated tenant partitions" }
        transition Dedicated -> Partitioned { trigger: "SF-ISOLATE" guard: "partitioning requested by the scheduler" }
        transition Partitioned -> Dedicated { trigger: "SF-ISOLATE" guard: "last partition released" }
    }
    mode Throttled { description: "Clocks capped by the thermal or power manager" }
    state Fault { description: "Isolated from the pool, awaiting operator action" }

    transition Off -> Boot { trigger: "SF-ATTEST" guard: "54 V bus on" }
    transition Boot -> Idle { trigger: "SF-ADMIT" guard: "attestation report verified" timing: 30 s }
    transition Boot -> Fault { trigger: "SF-ADMIT" guard: "measurement does not match the golden values" }
    transition Idle -> Compute { trigger: "SF-SCHED" guard: "kernel enqueued" }
    transition Compute -> Idle { trigger: "SF-SYNC" guard: "no pending work" }
    transition Compute -> Throttled { trigger: "SF-THROTTLE" guard: "Tj > 363 K or board power > 770 W" timing: 100 ms priority: 1 }
    transition Throttled -> Compute { trigger: "SF-SENSE" guard: "Tj < 353 K for 10 s" }
    transition Throttled -> Fault { trigger: "SF-SENSE" guard: "Tj > 378 K" priority: 1 }
    transition Compute -> Fault { trigger: "SF-ECC" guard: "uncorrectable error outside a tenant page" }
    transition Fault -> Off { trigger: "SF-CUT" guard: "operator power cycle" }
    transition Idle -> Off { trigger: "SF-CUT" guard: "leak detected or shutdown requested" }
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

scenario "Job launch and all-reduce step" {
    description: "A training job is dispatched, its tensors staged, and one gradient synchronization step runs over the fabric"
    participants: ["LC-ORCH", "LC-HOST", "LC-ACC", "LC-FSW", "LC-BMC"]
    message "LC-ORCH" -> "LC-HOST" "dispatch job" { type: "async" exchange: "Job dispatch" }
    message "LC-HOST" -> "LC-ACC" "DMA weights and activations (1 GB)" { type: "async" exchange: "Tensor transfers" timing: 18 ms }
    fragment loop "for each training step" {
        message "LC-HOST" -> "LC-ACC" "launch forward and backward kernels" { type: "async" exchange: "Kernel launches" timing: 20 us }
        message "LC-ACC" -> "LC-FSW" "ring all-reduce: send partial sums" { type: "async" exchange: "Collective traffic" timing: 1900 us }
        message "LC-FSW" -> "LC-ACC" "ring all-reduce: deliver reduced chunk" { type: "async" exchange: "Collective traffic" }
        message "LC-ACC" -> "LC-HOST" "step complete" { type: "async" exchange: "Step completion" timing: 10 us }
    }
    fragment par {
        operand "telemetry" {
            message "LC-ACC" -> "LC-BMC" "telemetry sample" { type: "async" exchange: "Telemetry stream" timing: 1 s }
        }
        operand "progress" {
            message "LC-HOST" -> "LC-ORCH" "checkpoint written" { type: "async" }
        }
    }
}

scenario "Thermal throttling event" {
    description: "A coolant flow drop raises the junction temperature; the device caps its clocks, the BMC raises the pump speed and tells the fleet"
    participants: ["LC-CLC", "LC-ACC", "LC-BMC", "LC-ORCH", "LC-HOST"]
    message "LC-ACC" -> "LC-BMC" "thermal event: Tj 365 K, clocks capped" { type: "async" exchange: "Telemetry stream" timing: 100 ms }
    message "LC-BMC" -> "LC-CLC" "raise pump speed, setpoint 303 K" { type: "sync" exchange: "Coolant setpoint" }
    message "LC-BMC" -> "LC-ORCH" "device throttled" { type: "async" exchange: "Fleet telemetry" timing: 1 s }
    message "LC-ORCH" -> "LC-HOST" "reduce batch size on the throttled device" { type: "async" exchange: "Job dispatch" }
    fragment alt {
        operand "temperature recovers below 353 K for 10 s" {
            message "LC-ACC" -> "LC-BMC" "clocks restored" { type: "async" exchange: "Telemetry stream" }
            message "LC-BMC" -> "LC-ORCH" "device nominal" { type: "async" exchange: "Fleet telemetry" }
        }
        operand "temperature keeps rising above 378 K" {
            message "LC-ACC" -> "LC-BMC" "fault: device isolated" { type: "async" exchange: "Telemetry stream" }
            message "LC-BMC" -> "LC-ORCH" "evict jobs, device out of pool" { type: "async" exchange: "Fleet telemetry" }
            message "LC-ORCH" -> "LC-HOST" "migrate the job" { type: "async" exchange: "Job dispatch" }
        }
    }
}

scenario "Attested boot" {
    description: "Power-on to admission: the device measures and signs its firmware, the BMC collects the evidence, the agent verifies it"
    participants: ["LC-BMC", "LC-ACC", "LC-ORCH"]
    message "LC-BMC" -> "LC-ACC" "SPDM GET_MEASUREMENTS with nonce" { type: "sync" exchange: "Device controls" }
    message "LC-ACC" -> "LC-BMC" "signed measurement report" { type: "sync" exchange: "Attestation reports" timing: 2 s }
    message "LC-BMC" -> "LC-ORCH" "attestation evidence of 8 devices" { type: "async" exchange: "Attestation evidence" }
    fragment alt {
        operand "all measurements match the golden values" {
            message "LC-ORCH" -> "LC-BMC" "admit tray to the pool" { type: "async" timing: 500 ms }
        }
        operand "a measurement differs" {
            message "LC-ORCH" -> "LC-BMC" "quarantine device, keep bus off" { type: "async" }
        }
    }
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

test_case "TC-PERF-001" {
    id: "TC-PERF-001" method: "test" verifies: ["REQ-PERF-001"]
    description: "Allocate 141 GB in one process on each of the 8 devices and touch every page"
    expected: "Allocation succeeds; no page fault; ECC counters unchanged"
}
test_case "TC-PERF-002" {
    id: "TC-PERF-002" method: "test" verifies: ["REQ-PERF-002"]
    description: "Streaming read benchmark over 128 GB, 10 runs, median reported"
    expected: "Median bandwidth >= 4.8 TB/s"
}
test_case "TC-PERF-003" {
    id: "TC-PERF-003" method: "test" verifies: ["REQ-PERF-003"]
    description: "Ring all-reduce of a 1 GB BF16 buffer across 8 devices, 1000 iterations, p99 latency"
    expected: "p99 <= 2.5 ms; bus bandwidth >= 850 GB/s"
}
test_case "TC-PERF-004" {
    id: "TC-PERF-004" method: "test" verifies: ["REQ-PERF-004"]
    description: "Pinned host-to-device copies of 1 GB, 100 runs"
    expected: "Sustained >= 56 GB/s"
}
test_case "TC-THM-001" {
    id: "TC-THM-001" method: "test" verifies: ["REQ-THM-001", "REQ-OBS-002"]
    description: "Thermal chamber: coolant flow reduced until Tj crosses 363 K; time from the crossing to the clock cap measured on the sensor bus"
    expected: "Cap applied within 100 ms; event timestamped and received by the agent within 1 s"
}
test_case "TC-THM-002" {
    id: "TC-THM-002" method: "test" verifies: ["REQ-THM-002"]
    description: "Full-load training run of 24 h at 308 K inlet and 1.5 L/min per module"
    expected: "No throttling event; Tj <= 358 K"
}
test_case "TC-THM-003" {
    id: "TC-THM-003" method: "test" verifies: ["REQ-THM-003"]
    description: "Water injected on the leak rope of slot 3; bus voltage of slot 3 recorded"
    expected: "Slot 3 bus off within 500 ms; other slots unaffected"
}
test_case "TC-PWR-001" {
    id: "TC-PWR-001" method: "test" verifies: ["REQ-PWR-001", "REQ-PWR-002"]
    description: "Power viruses on all 8 devices; shelf and module power logged at 10 kHz"
    expected: "Module <= 700 W average; excursion above 770 W braked within 1 ms; tray <= 10.2 kW"
}
test_case "TC-RAS-001" {
    id: "TC-RAS-001" method: "test" verifies: ["REQ-RAS-001"]
    description: "Fault injection of double-bit errors in a tenant page and in a kernel page"
    expected: "Tenant job aborts alone, page retired; kernel page error drives Fault"
}
test_case "TC-RAS-002" {
    id: "TC-RAS-002" method: "analysis" verifies: ["REQ-RAS-002"]
    description: "Reliability prediction from component failure rates and the HALT campaign at 308 K"
    expected: "MTBF >= 1,000,000 h"
}
test_case "TC-SEC-001" {
    id: "TC-SEC-001" method: "test" verifies: ["REQ-SEC-001"]
    description: "Boot with a modified thermal firmware stage; SPDM evidence and admission observed"
    expected: "Measurement mismatch reported; device not admitted; bus kept off"
}
test_case "TC-SEC-002" {
    id: "TC-SEC-002" method: "test" verifies: ["REQ-SEC-002"]
    description: "Two partitions: a prime-and-probe attacker and a victim running matrix kernels on known weights"
    expected: "No statistically significant leak over 10^6 probes"
}
test_case "TC-SEC-003" {
    id: "TC-SEC-003" method: "inspection" verifies: ["REQ-SEC-003"]
    description: "Inspection of the switch firmware configuration and of the key rotation logs over 7 days"
    expected: "AES-GCM enabled on every scale-out port; rotation interval <= 24 h"
}
test_case "TC-OBS-001" {
    id: "TC-OBS-001" method: "test" verifies: ["REQ-OBS-001"]
    description: "Telemetry capture on the management bus for 1 h"
    expected: "3600 samples per device, no gap, all fields populated"
}

// ---------------------------------------------------------------------------
// Constraints
// ---------------------------------------------------------------------------

constraint "All-reduce step within budget" {
    id: "CST-ALLREDUCE"
    assert: ("SF-SCHED".latency + "SF-REDUCE".latency + "SF-ALLRED".latency + "SF-SYNC".latency) <= "FC-ALLREDUCE".latency_budget
}
constraint "Throttling within budget" {
    id: "CST-THROTTLE"
    assert: ("SF-SENSE".latency + "SF-THROTTLE".latency + "SF-TELEM".latency) <= "FC-THROTTLE".latency_budget
}
constraint "Tray power within the shelf rating" {
    id: "CST-POWER"
    assert: (8 * "PN-OAM".power + "PN-HOST".power + "PN-BASE".power) <= "PN-PSHELF".power
}
constraint "HBM capacity" {
    id: "CST-HBM"
    assert: 6 * "LC-HBM".ram >= "LC-ACC".ram
}
constraint "Host link sustains the DMA rate" {
    id: "CST-PCIE"
    assert: "PL-PCIE-HOST".bandwidth >= 448 Gbps
}

// ---------------------------------------------------------------------------
// Traceability
// ---------------------------------------------------------------------------

trace "LC-ACC" satisfies "REQ-PERF-001" { rationale: "141 GB of HBM3e per device" }
trace "LC-MEM" satisfies "REQ-PERF-002" { rationale: "Six HBM3e stacks on a 6144-bit interface" }
trace "LC-HBM" satisfies "REQ-PERF-001" { rationale: "24 GB per stack, six stacks" }
trace "LC-FAB" satisfies "REQ-PERF-003" { rationale: "900 GB/s per device, ring collectives in hardware" }
trace "LC-FSW" satisfies "REQ-PERF-003" { rationale: "Non-blocking switch across the 8 devices" }
trace "LC-HIF" satisfies "REQ-PERF-004" { rationale: "PCIe Gen5 x16 endpoint, 64 GB/s raw" }
trace "LC-THM" satisfies "REQ-THM-001" { rationale: "Sensors at 10 kHz, cap within 5 ms" }
trace "LC-CLC" satisfies "REQ-THM-002" { rationale: "Holds the inlet at 308 K" }
trace "LC-CLC" satisfies "REQ-THM-003" { rationale: "Leak rope monitoring" }
trace "LC-BMC" satisfies "REQ-THM-003" { rationale: "Cuts the 54 V bus of the module" }
trace "LC-PDN" satisfies "REQ-PWR-001" { rationale: "700 W TDP regulator design" }
trace "LC-PDN" satisfies "REQ-PWR-002" { rationale: "Hardware power brake within 500 us" }
trace "LC-MEM" satisfies "REQ-RAS-001" { rationale: "SECDED ECC and page retirement" }
trace "LC-ACC" satisfies "REQ-RAS-002" { rationale: "MTBF 1,000,000 h at 308 K inlet" }
trace "LC-SEC" satisfies "REQ-SEC-001" { rationale: "Measured boot and SPDM responder" }
trace "LC-ATT" satisfies "REQ-SEC-001" { rationale: "SPDM requester with fresh nonces" }
trace "LC-ORCH" satisfies "REQ-SEC-001" { rationale: "Admission only after verification" }
trace "LC-CDIE" satisfies "REQ-SEC-002" { rationale: "Address-space and cache-way partitioning" }
trace "LC-SEC" satisfies "REQ-SEC-002" { rationale: "Fabric partitioning per tenant" }
trace "LC-FSW" satisfies "REQ-SEC-003" { rationale: "AES-GCM on scale-out links, 24 h rotation" }
trace "LC-ACC" satisfies "REQ-OBS-001" { rationale: "Telemetry at 1 Hz" }
trace "LC-BMC" satisfies "REQ-OBS-001" { rationale: "Aggregation at 1 Hz" }
trace "LC-BMC" satisfies "REQ-OBS-002" { rationale: "Events forwarded within 1 s" }
trace "LC-ORCH" satisfies "REQ-OBS-002" { rationale: "Fleet export" }
trace "LC-HOST" satisfies "REQ-PERF-004" { rationale: "Pinned staging buffers for DMA" }
trace "LC-RT" satisfies "REQ-PERF-003" { rationale: "Ring schedule planned per step" }
trace "LC-DRV" satisfies "REQ-PERF-004" { rationale: "Maps device rings and doorbells" }
trace "LC-TC" satisfies "REQ-PERF-002" { rationale: "Consumes the HBM bandwidth" }
trace "LC-SMC" satisfies "REQ-PERF-003" { rationale: "Local reduce in L2 before the ring" }

trace "SF-SCHED" realizes "OA-ACT-TRAIN"
trace "SF-EXEC" realizes "OA-ACT-TRAIN"
trace "SF-ALLRED" realizes "OA-ACT-TRAIN"
trace "SF-ISOLATE" realizes "OA-ACT-SERVE"
trace "SF-THROTTLE" realizes "OA-ACT-ENVELOPE"
trace "SF-POWERCAP" realizes "OA-ACT-ENVELOPE"
trace "SF-ATTEST" realizes "OA-ACT-ATTEST"
trace "SF-ADMIT" realizes "OA-ACT-ADMIT"
trace "SF-TELEM" realizes "OA-ACT-MONITOR"

// ---------------------------------------------------------------------------
// Diagrams
// ---------------------------------------------------------------------------

diagram "Accelerator internals" {
    view: lab
    description: "The eight sub-components of one accelerator"
    shows: ["LC-CDIE", "LC-TC", "LC-SMC", "LC-MEM", "LC-HBM", "LC-FAB", "LC-HIF", "LC-PDN", "LC-THM", "LC-SEC"]
}

diagram "Thermal and power control loop" {
    view: lab
    description: "From the device sensors to the coolant loop and the fleet"
    shows: ["LC-ACC", "LC-BMC", "LC-CLC", "LC-ORCH"]
}

diagram "Compute tray wiring" {
    view: pab
    description: "Nodes and links of one tray: PCIe, fabric, management, power, coolant"
    shows: ["PN-HOST", "PN-OAM", "PN-BASE", "PN-BMC", "PN-PSHELF", "PN-CDU", "PN-RACKSW"]
}
