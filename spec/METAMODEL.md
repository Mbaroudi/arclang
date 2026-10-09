# ArcLang metamodel

Language version **4.0.0**. GENERATED from `src/compiler/metamodel.rs` by `arclang metamodel --format markdown`; a golden test fails when this file and the compiler disagree. Do not edit by hand.

The metamodel is independent of Capella: it names the Arcadia metaclass each kind corresponds to, and the SysML v2 construct the exporter maps it to. Attribute values are typed; the compiler reports type violations as `metamodel:` warnings and the production gate turns them into blockers.

## Element kinds

### Operational Analysis

#### Actor

Human or external system interacting with the system.

- Arcadia: `OperationalActor`
- SysML v2: `part def (actor)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `category` | `text` | Actor category. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### OperationalEntity

Organisation or system of the operational world.

- Arcadia: `Entity`
- SysML v2: `part def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `type` | `text` | Component category (free text). |
| `layer` | `text` | Overrides the architectural layer label. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |

#### OperationalCapability

Expected ability of the operational world. May be declared inside another operational capability.

- Arcadia: `OperationalCapability`
- SysML v2: `use case def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `involves` | `list<ref<Element>>` | Actors, entities, activities and processes involved. |
| `extends` | `list<ref<OperationalCapability>>` | Capabilities of the same level this one extends. |
| `includes` | `list<ref<OperationalCapability>>` | Capabilities of the same level this one includes. |
| `specializes` | `list<ref<OperationalCapability>>` | Capabilities of the same level this one is a special case of. |

#### OperationalActivity

Activity performed by an entity or actor.

- Arcadia: `OperationalActivity`
- SysML v2: `action def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `category` | `text` | Functional category (free text). |
| `inputs` | `list<text>` | Input data names. |
| `outputs` | `list<text>` | Output data names. |
| `latency` | `quantity<time>` | Execution latency; summed along functional chains by the gate. |
| `execution_time` | `quantity<time>` | Alias of latency. |
| `wcet` | `quantity<time>` | Worst-case execution time. |
| `period` | `quantity<time>` | Activation period. |
| `deadline` | `quantity<time>` | Completion deadline. |
| `frequency` | `quantity<frequency>` | Activation frequency. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### OperationalExchange

Interaction between operational entities.

- Arcadia: `CommunicationMean / OperationalExchange`
- SysML v2: `connect`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<Element>` | Source element or port. |
| `to` | `ref<Element>` | Target element or port. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |

#### OperationalProcess

Ordered path of activities fulfilling a capability.

- Arcadia: `OperationalProcess`
- SysML v2: `action def (sequence)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `involves` | `list<ref<Element>>` | Ordered functions/exchanges of the chain. |
| `latency_budget` | `quantity<time>` | End-to-end budget checked by the gate. |
| `capability` | `ref<Capability>` | Capability this chain exemplifies. |

### System Analysis

#### Requirement

Stakeholder, system or safety requirement.

- Arcadia: `Requirement`
- SysML v2: `requirement`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `priority` | `enum Priority` | Business priority. |
| `category` | `text` | Requirement category (functional, performance, safety...). |
| `rationale` | `text` | Why the requirement exists. |
| `source` | `text` | Origin document or stakeholder. |
| `status` | `text` | Lifecycle status. |
| `verification_method` | `enum VerificationMethod` | Intended verification method. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### Mission

High-level goal the system contributes to.

- Arcadia: `Mission`
- SysML v2: `use case def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### Capability

Expected ability of the system. May be declared inside another capability.

- Arcadia: `Capability`
- SysML v2: `use case def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `mission` | `ref<Mission>` | Mission this capability contributes to. |
| `realizes` | `ref<OperationalCapability>` | Operational capability realized. |
| `involves` | `list<ref<Element>>` | Functions, actors and chains involved. |
| `extends` | `list<ref<Capability>>` | Capabilities of the same level this one extends. |
| `includes` | `list<ref<Capability>>` | Capabilities of the same level this one includes. |
| `specializes` | `list<ref<Capability>>` | Capabilities of the same level this one is a special case of. |

#### SystemFunction

Function the system performs.

- Arcadia: `SystemFunction`
- SysML v2: `action def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `category` | `text` | Functional category (free text). |
| `inputs` | `list<text>` | Input data names. |
| `outputs` | `list<text>` | Output data names. |
| `latency` | `quantity<time>` | Execution latency; summed along functional chains by the gate. |
| `execution_time` | `quantity<time>` | Alias of latency. |
| `wcet` | `quantity<time>` | Worst-case execution time. |
| `period` | `quantity<time>` | Activation period. |
| `deadline` | `quantity<time>` | Completion deadline. |
| `frequency` | `quantity<frequency>` | Activation frequency. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### FunctionPort

Oriented port of a function (strictly in XOR out).

- Arcadia: `FunctionInputPort / FunctionOutputPort`
- SysML v2: `in/out item`

| Attribute | Type | Meaning |
|---|---|---|
| `data_type` | `text` | Exchanged data type. |
| `type` | `enum PortKind` | data | control | event. |

#### FunctionalExchange

Data flow between function ports.

- Arcadia: `FunctionalExchange`
- SysML v2: `flow`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<Element>` | Source element or port. |
| `to` | `ref<Element>` | Target element or port. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |

#### SystemActor

External actor at system level.

- Arcadia: `SystemActor`
- SysML v2: `part def (actor)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### SystemComponent

The system or one of its external components.

- Arcadia: `SystemComponent`
- SysML v2: `part def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `type` | `text` | Component category (free text). |
| `layer` | `text` | Overrides the architectural layer label. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |

#### FunctionalChain

Ordered functions and exchanges realizing one dataflow path.

- Arcadia: `FunctionalChain`
- SysML v2: `action def (sequence)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `involves` | `list<ref<Element>>` | Ordered functions/exchanges of the chain. |
| `latency_budget` | `quantity<time>` | End-to-end budget checked by the gate. |
| `capability` | `ref<Capability>` | Capability this chain exemplifies. |

### Logical Architecture

#### LogicalComponent

Behavioural building block, may be nested.

- Arcadia: `LogicalComponent`
- SysML v2: `part def + part`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `type` | `text` | Component category (free text). |
| `layer` | `text` | Overrides the architectural layer label. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |
| `multiplicity` | `multiplicity` | How many of the element its owner has: a count (`4`) or a range as text (`"0..1"`, `"1..*"`, `"*"`). One when absent. |

#### LogicalFunction

Function allocated to a logical component.

- Arcadia: `LogicalFunction`
- SysML v2: `action def (perform)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `category` | `text` | Functional category (free text). |
| `inputs` | `list<text>` | Input data names. |
| `outputs` | `list<text>` | Output data names. |
| `latency` | `quantity<time>` | Execution latency; summed along functional chains by the gate. |
| `execution_time` | `quantity<time>` | Alias of latency. |
| `wcet` | `quantity<time>` | Worst-case execution time. |
| `period` | `quantity<time>` | Activation period. |
| `deadline` | `quantity<time>` | Completion deadline. |
| `frequency` | `quantity<frequency>` | Activation frequency. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### ComponentPort

Oriented port of a component (in, out, inout).

- Arcadia: `ComponentPort`
- SysML v2: `port`

| Attribute | Type | Meaning |
|---|---|---|
| `interface` | `text` | Interface type carried. |
| `protocol` | `text` | Protocol. |

#### LogicalInterface

Contract between two components.

- Arcadia: `Interface`
- SysML v2: `interface def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<LogicalComponent>` | Provider. |
| `to` | `ref<LogicalComponent>` | Consumer. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |

#### ComponentExchange

Exchange between component ports.

- Arcadia: `ComponentExchange`
- SysML v2: `connect`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<Element>` | Source element or port. |
| `to` | `ref<Element>` | Target element or port. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |

#### CapabilityRealization

Realization of a system capability by components. May be declared inside another realization.

- Arcadia: `CapabilityRealization`
- SysML v2: `use case def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `realizes` | `ref<Capability>` | Capability realized. |
| `involves` | `list<ref<Element>>` | Components and chains involved. |
| `extends` | `list<ref<CapabilityRealization>>` | Capabilities of the same level this one extends. |
| `includes` | `list<ref<CapabilityRealization>>` | Capabilities of the same level this one includes. |
| `specializes` | `list<ref<CapabilityRealization>>` | Capabilities of the same level this one is a special case of. |

### Physical Architecture

#### PhysicalNode

Hardware node hosting behaviour components.

- Arcadia: `PhysicalComponent (NODE)`
- SysML v2: `part def + part`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `type` | `text` | Component category (free text). |
| `layer` | `text` | Overrides the architectural layer label. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |
| `multiplicity` | `multiplicity` | How many of the element its owner has: a count (`4`) or a range as text (`"0..1"`, `"1..*"`, `"*"`). One when absent. |
| `processor` | `text` | Processor. |
| `cpu` | `text` | CPU description. |
| `redundancy` | `text` | Redundancy scheme. |

#### PhysicalPort

Unoriented physical connector.

- Arcadia: `PhysicalPort`
- SysML v2: `port`

| Attribute | Type | Meaning |
|---|---|---|
| `connector` | `text` | Connector type. |

#### BehaviorComponent

Software/behaviour deployed on a node.

- Arcadia: `PhysicalComponent (BEHAVIOR)`
- SysML v2: `part`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `safety_level` | `enum SafetyLevel` | ASIL, DAL or SIL assigned to the element. |
| `asil` | `enum Asil` | ISO 26262 ASIL (QM, ASIL-A..D). |
| `dal` | `enum Dal` | DO-178C design assurance level (DAL-A..E). |

#### HardwareComponent

Hardware part of a node.

- Arcadia: `PhysicalComponent (NODE, nested)`
- SysML v2: `part`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |

#### PhysicalLink

Physical medium between nodes or ports.

- Arcadia: `PhysicalLink`
- SysML v2: `connect (binding)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<Element>` | Source element or port. |
| `to` | `ref<Element>` | Target element or port. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |

#### PhysicalExchange

Message routed over a link.

- Arcadia: `ComponentExchange (physical)`
- SysML v2: `flow`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `from` | `ref<Element>` | Source element or port. |
| `to` | `ref<Element>` | Target element or port. |
| `protocol` | `text` | Transport protocol (CAN FD, Ethernet, ARINC 429...). |
| `bandwidth` | `quantity<data rate>` | Available or required data rate. |
| `latency` | `quantity<time>` | Transport latency. |
| `frequency` | `quantity<frequency>` | Message rate. |
| `period` | `quantity<time>` | Message period. |
| `size` | `quantity<data size>` | Payload size. |
| `data_type` | `text` | Exchanged data type. |
| `via` | `ref<PhysicalLink>` | Carrying link. |
| `message_type` | `text` | Message/frame type. |

#### Deployment

Allocation of a logical component to a node.

- Arcadia: `ComponentDeploymentLink`
- SysML v2: `allocate`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### PhysicalPath

Ordered links routing an exchange.

- Arcadia: `PhysicalPath`
- SysML v2: `connect (sequence)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `involves` | `list<ref<PhysicalLink>>` | Ordered links. |

### EPBS

#### EpbsSystem

Top configuration item.

- Arcadia: `ConfigurationItem (SYSTEM)`
- SysML v2: `part def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### EpbsSubsystem

Subsystem configuration item.

- Arcadia: `ConfigurationItem (CS)`
- SysML v2: `part def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### EpbsItem

Leaf configuration item or assembly.

- Arcadia: `ConfigurationItem (HW/SW)`
- SysML v2: `part def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `part_number` | `text` | Part number. |
| `supplier` | `text` | Supplier. |
| `memory` | `text` | Memory configuration, descriptive (`"2 GB DDR4, 64 GB eMMC"`). Use `ram`/`flash`/`storage` for typed sizes. |
| `ram` | `quantity<data size>` | RAM capacity. |
| `flash` | `quantity<data size>` | Flash capacity. |
| `storage` | `quantity<data size>` | Mass storage capacity. |
| `power` | `quantity<power>` | Power consumption. |
| `voltage` | `quantity<voltage>` | Supply voltage. |
| `mass` | `quantity<mass>` | Mass. |
| `weight` | `quantity<mass>` | Alias of mass. |

### Data model

#### Class

Structured data element; every attribute but `id` and `description` is a field, kept in declared order.

- Arcadia: `Class`
- SysML v2: `attribute def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### Enumeration

Enumerated data type.

- Arcadia: `Enumeration`
- SysML v2: `enum def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `values` | `list<text>` | Literals. |

#### DataType

Primitive data type.

- Arcadia: `DataType`
- SysML v2: `attribute def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `base` | `text` | Base type. |
| `unit` | `enum Unit` | Unit symbol of the values. |

#### ExchangeItem

Set of data elements exchanged together.

- Arcadia: `ExchangeItem`
- SysML v2: `item def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `mechanism` | `enum ExchangeMechanism` | Exchange mechanism. |
| `elements` | `list<ref<Class>>` | Grouped data elements. |

### Transverse

#### Hazard

HARA entry: hazardous event and its classification.

- Arcadia: `(safety extension)`
- SysML v2: `requirement (hazard)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `severity` | `enum HazardSeverity` | ISO 26262 S0-S3 or DO-178C failure condition. |
| `exposure` | `enum Exposure` | ISO 26262 E0-E4. |
| `controllability` | `enum Controllability` | ISO 26262 C0-C3. |
| `condition` | `enum FailureCondition` | DO-178C failure condition class. |
| `asil` | `enum Asil` | Declared ASIL; must match S/E/C. |
| `dal` | `enum Dal` | Declared DAL; must match the condition. |
| `mitigated_by` | `list<ref<Requirement>>` | Safety requirements mitigating the hazard. |

#### FmeaEntry

FMEA line: failure mode with S/O/D rating.

- Arcadia: `(safety extension)`
- SysML v2: `requirement (failure mode)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `failure_mode` | `text` | Failure mode. |
| `effect` | `text` | Effect. |
| `cause` | `text` | Cause. |
| `severity` | `text` | Severity: rating, class (S3) or description — free text by design, FMEA practices differ. |
| `occurrence` | `text` | Occurrence: rating, class or rate description. |
| `detection` | `text` | Detection: rating, class or detection means. |
| `rpn` | `number` | Risk priority number. |

#### Trace

Typed link between two elements.

- Arcadia: `AbstractTrace / Realization`
- SysML v2: `satisfy / verify / allocate`

| Attribute | Type | Meaning |
|---|---|---|
| `from` | `ref<Element>` | Source. |
| `to` | `ref<Element>` | Target. |
| `type` | `enum TraceKind` | Link kind. |
| `rationale` | `text` | Justification. |

#### TestCase

Verification case covering requirements.

- Arcadia: `(V&V extension)`
- SysML v2: `verification def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `verifies` | `list<ref<Requirement>>` | Requirements verified. |
| `method` | `enum VerificationMethod` | Verification method. |
| `procedure` | `text` | Procedure. |
| `expected` | `text` | Expected result. |

#### Model

The model itself: root of the element graph, owner of every top-level element.

- Arcadia: `Project / SystemEngineering`
- SysML v2: `package`

| Attribute | Type | Meaning |
|---|---|---|
| `name` | `text` | Model name. |
| `version` | `text` | Model version. |
| `description` | `text` | Free-text description. |

#### Type

Reusable definition: `type Name extends Base { ... }`. Declares typed attributes, ports and the attributes instances must provide. Redefinitions must keep the dimension. Types have their own namespace.

- Arcadia: `(extension — closest Capella notion: REC/RPL)`
- SysML v2: `abstract part def / action def, specialized with :>`

| Attribute | Type | Meaning |
|---|---|---|
| `required` | `list<text>` | Attributes every instance must provide (declared or inherited). |
| `description` | `text` | Free-text description (not inherited). |

#### Constraint

Dimension-checked comparison over typed attributes (`assert: <expression>`). Ill-formed is a compile error; violated is a warning and a gate blocker.

- Arcadia: `Constraint`
- SysML v2: `assert constraint`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### StateMachine

Modes and states of an element.

- Arcadia: `StateMachine`
- SysML v2: `state def`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `initial` | `ref<State>` | Initial state. |

#### State

State (undergone) or mode (chosen behaviour).

- Arcadia: `State / Mode`
- SysML v2: `state`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |

#### Transition

Transition between states.

- Arcadia: `StateTransition`
- SysML v2: `transition`

| Attribute | Type | Meaning |
|---|---|---|
| `trigger` | `text` | Triggering event. |
| `guard` | `text` | Guard condition. |
| `action` | `text` | Effect. |
| `timing` | `quantity<time>` | Timing constraint. |
| `priority` | `number` | Priority among concurrent transitions. |

#### Scenario

Sequence of messages between participants.

- Arcadia: `Scenario`
- SysML v2: `interaction (sequence)`

| Attribute | Type | Meaning |
|---|---|---|
| `id` | `identifier` | Stable identity; drives the UUIDv5. Defaults to the name. |
| `name` | `text` | Display name; overrides the block name. |
| `description` | `text` | Free-text description. |
| `is` | `ref<Type>` | User-defined type of the element, or a list of types; their attributes (and ports, for components) are inherited. |
| `participants` | `list<ref<Element>>` | Lifelines. |

#### Message

Message between two participants.

- Arcadia: `SequenceMessage`
- SysML v2: `message`

| Attribute | Type | Meaning |
|---|---|---|
| `type` | `enum MessageKind` | sync | async. |
| `timing` | `quantity<time>` | Timing constraint. |

## Relationships

The element graph served by the Systems Modeling API (`/api/systems-modeling`) is made of the kinds above and of these relationship types. A relationship whose end does not resolve is reported in the commit, never dropped.

| Relationship | Source | Target | Meaning |
|---|---|---|---|
| `Trace` | element | element | A `trace` declaration; `traceKind` is one of the TraceKind values. |
| `Verification` | TestCase | Requirement | `verifies:` of a test case. |
| `Mitigation` | Requirement | Hazard | `mitigated_by:` of a hazard. |
| `Typing` | element or PhysicalLink | Type | `is:` — the element is an instance of the type. |
| `Specialization` | Type | Type | `extends` — the source type specializes the target. |
| `Involvement` | chain, path, capability | element | `involves:`; `order` is the 1-based position. |
| `Realization` | capability | capability | `realizes:` across layers. |
| `CapabilityExtend` | capability | capability of the same level | `extends:` of a capability. |
| `CapabilityInclude` | capability | capability of the same level | `includes:` of a capability. |
| `CapabilityGeneralization` | capability | capability of the same level | `specializes:` of a capability. |
| `Contribution` | Capability | Mission | `mission:` of a capability. |
| `Deployment` | LogicalComponent | PhysicalNode | `deploys` / `deployment`. |
| `OperationalExchange` | operational entity | operational entity | Interaction or communication means. |
| `FunctionalExchange` | FunctionPort or function | FunctionPort or function | Data flow between functions. |
| `ComponentExchange` | ComponentPort or component | ComponentPort or component | Exchange between components. |
| `LogicalInterface` | LogicalComponent | LogicalComponent | Interface contract between two components. |
| `PhysicalLink` | PhysicalNode | PhysicalNode | Physical medium; carries typed attributes and may be typed. |
| `PhysicalExchange` | PhysicalNode | PhysicalNode | Message routed over a link. |
| `Transition` | State | State | State machine transition. |
| `Message` | element | element | Scenario message; `order` is the 1-based position. |

## Enumerations

Values match ignoring case and the separators `-`, `_` and space.

| Enumeration | Values | Meaning |
|---|---|---|
| Asil | `QM`, `ASIL-A`, `ASIL-B`, `ASIL-C`, `ASIL-D` | ISO 26262 automotive safety integrity level. |
| Dal | `DAL-A`, `DAL-B`, `DAL-C`, `DAL-D`, `DAL-E` | DO-178C design assurance level. |
| SafetyLevel | `QM`, `ASIL-A`, `ASIL-B`, `ASIL-C`, `ASIL-D`, `DAL-A`, `DAL-B`, `DAL-C`, `DAL-D`, `DAL-E`, `SIL-1`, `SIL-2`, `SIL-3`, `SIL-4` | Any integrity level: ASIL (ISO 26262), DAL (DO-178C) or SIL (IEC 61508). |
| Priority | `Critical`, `High`, `Medium`, `Low` | Requirement priority. |
| HazardSeverity | `S0`, `S1`, `S2`, `S3`, `Catastrophic`, `Hazardous`, `Major`, `Minor`, `No Effect` | ISO 26262 severity class or DO-178C failure condition. |
| Exposure | `E0`, `E1`, `E2`, `E3`, `E4` | ISO 26262 exposure class. |
| Controllability | `C0`, `C1`, `C2`, `C3` | ISO 26262 controllability class. |
| FailureCondition | `Catastrophic`, `Hazardous`, `Major`, `Minor`, `No Effect` | DO-178C failure condition classification. |
| VerificationMethod | `test`, `analysis`, `inspection`, `demonstration` | Verification method. |
| ExchangeMechanism | `EVENT`, `FLOW`, `OPERATION`, `DATA`, `SHARED_DATA` | Arcadia exchange item mechanism. |
| TraceKind | `satisfies`, `implements`, `validates`, `verifies`, `realizes`, `refines`, `allocates` | Traceability link kinds. |
| PortKind | `data`, `control`, `event` | Function port nature. |
| MessageKind | `sync`, `async` | Scenario message kind. |
| Unit | (unit table) | Any symbol of the unit table below. |

## Units

A quantity is `<number> <unit>` (`latency: 25 ms`). Symbols are case-sensitive. Each dimension converts to its canonical unit for arithmetic.

| Symbol | Dimension | Factor to canonical | SysML v2 (SI) |
|---|---|---|---|
| `ns` | time (s) | 0.000000001 | `ns` |
| `us` | time (s) | 0.000001 | `us` |
| `µs` | time (s) | 0.000001 | `us` |
| `ms` | time (s) | 0.001 | `ms` |
| `s` | time (s) | 1 | `s` |
| `min` | time (s) | 60 | `min` |
| `h` | time (s) | 3600 | `h` |
| `Hz` | frequency (Hz) | 1 | `Hz` |
| `kHz` | frequency (Hz) | 1000 | `kHz` |
| `MHz` | frequency (Hz) | 1000000 | `MHz` |
| `GHz` | frequency (Hz) | 1000000000 | `GHz` |
| `bps` | data rate (bit/s) | 1 | `'bit/s'` |
| `bit/s` | data rate (bit/s) | 1 | `'bit/s'` |
| `kbps` | data rate (bit/s) | 1000 | `'kbit/s'` |
| `Kbps` | data rate (bit/s) | 1000 | `'kbit/s'` |
| `kbit/s` | data rate (bit/s) | 1000 | `'kbit/s'` |
| `Mbps` | data rate (bit/s) | 1000000 | `'Mbit/s'` |
| `Mbit/s` | data rate (bit/s) | 1000000 | `'Mbit/s'` |
| `Gbps` | data rate (bit/s) | 1000000000 | `'Gbit/s'` |
| `Gbit/s` | data rate (bit/s) | 1000000000 | `'Gbit/s'` |
| `bit` | data size (bit) | 1 | `bit` |
| `B` | data size (bit) | 8 | `B` |
| `kB` | data size (bit) | 8000 | `kB` |
| `KB` | data size (bit) | 8000 | `kB` |
| `MB` | data size (bit) | 8000000 | `MB` |
| `GB` | data size (bit) | 8000000000 | `GB` |
| `KiB` | data size (bit) | 8192 | `KiB` |
| `MiB` | data size (bit) | 8388608 | `MiB` |
| `GiB` | data size (bit) | 8589934592 | `GiB` |
| `mm` | length (m) | 0.001 | `mm` |
| `cm` | length (m) | 0.01 | `cm` |
| `m` | length (m) | 1 | `m` |
| `km` | length (m) | 1000 | `km` |
| `g` | mass (kg) | 0.001 | `g` |
| `kg` | mass (kg) | 1 | `kg` |
| `t` | mass (kg) | 1000 | `t` |
| `N` | force (N) | 1 | `N` |
| `kN` | force (N) | 1000 | `kN` |
| `mV` | voltage (V) | 0.001 | `mV` |
| `V` | voltage (V) | 1 | `V` |
| `kV` | voltage (V) | 1000 | `kV` |
| `mA` | electric current (A) | 0.001 | `mA` |
| `A` | electric current (A) | 1 | `A` |
| `mW` | power (W) | 0.001 | `mW` |
| `W` | power (W) | 1 | `W` |
| `kW` | power (W) | 1000 | `kW` |
| `J` | energy (J) | 1 | `J` |
| `kJ` | energy (J) | 1000 | `kJ` |
| `Wh` | energy (J) | 3600 | `'W*h'` |
| `kWh` | energy (J) | 3600000 | `'kW*h'` |
| `Pa` | pressure (Pa) | 1 | `Pa` |
| `kPa` | pressure (Pa) | 1000 | `kPa` |
| `bar` | pressure (Pa) | 100000 | `bar` |
| `K` | temperature (K) | 1 | `K` |
| `m/s` | speed (m/s) | 1 | `'m/s'` |
| `km/h` | speed (m/s) | 0.2777777777777778 | `'km/h'` |
| `percent` | ratio (one) | 0.01 | `'%'` |
| `%` | ratio (one) | 0.01 | `'%'` |

## Traceability rules

Dangling endpoints are compile errors. Kind mismatches are `metamodel:` warnings.

| Kind | From | To | Meaning |
|---|---|---|---|
| `satisfies` | component, function, node, actor | Requirement | Architecture element satisfies a requirement. Counted by the gate. |
| `implements` | component, node | function, Requirement | Element implements a function or requirement. |
| `verifies` | TestCase | Requirement | Verification case covers a requirement (also via `verifies:`). |
| `validates` | TestCase, Scenario | Requirement, Capability | Validation evidence. |
| `realizes` | lower-layer element | upper-layer element | Inter-layer realization: SA→OA, LA→SA, PA→LA, EPBS→PA. |
| `refines` | Requirement | Requirement | Requirement decomposition. |
| `allocates` | component, node | function, component | Allocation (also expressed by nesting and `deployment`). |
