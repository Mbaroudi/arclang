// Public DC fast-charging station (150-350 kW) with its charge point operator
// back end, through the five Arcadia levels. Written to compile with zero
// findings and to pass the production gate (IEC 61508): every requirement is
// satisfied by an element and verified by a test case, every hazard is
// classified consistently, every functional chain holds its latency budget,
// every exchange between nodes is carried by a physical link.
//
//   arclang check examples/energy/ev_charging_station.arc
//   arclang gate --standard iec61508 examples/energy/ev_charging_station.arc
//   arclang workbench examples/energy/ev_charging_station.arc

model EvChargingStation {
  name: "DC Fast-Charging Station"
  description: "Eight 350 kW CCS2 dispensers on a 630 kVA grid connection; ISO 15118 plug-and-charge, OCPP 2.0.1 to the CSMS"
  version: "1.0"
}

// ---- Reusable types -------------------------------------------------------
// A safety function must state the latency the chain budgets sum; a power
// module carries its ratings, inherited by the components typed with it.

type "Safety function" {
  description: "Function of the protection path"
  required: ["latency"]
  safety_level: "SIL-2"
}
type "DC power module" {
  description: "Isolated AC/DC converter brick, paralleled to reach the connector rating"
  power: 50 kW
  voltage: 1000 V
  efficiency: 97 %
}

// ---- Requirements ---------------------------------------------------------

requirements system {
  requirement "REQ-CHG-001" {
    description: "Each CCS2 connector shall deliver up to 350 kW DC at 1000 V and 500 A through a liquid-cooled cable"
    category: "performance"  priority: "Critical"  source: "Product sheet, IEC 61851-23"  verification_method: "test"
  }
  requirement "REQ-CHG-002" {
    description: "A session shall be authorized by ISO 15118-2 plug-and-charge (contract certificate) without any driver action; RFID and payment card remain as fallbacks"
    category: "functional"  priority: "High"  source: "ISO 15118-2, CPO service definition"  verification_method: "test"
  }
  requirement "REQ-CHG-003" {
    description: "A driver without an account shall pay with a contactless EMV card at the terminal; the pre-authorization is settled on the metered energy"
    category: "functional"  priority: "High"  source: "AFIR article 5"  verification_method: "demonstration"
  }
  requirement "REQ-CHG-004" {
    description: "Energy shall flow within 10 s of the authorization of the session"
    category: "performance"  priority: "Medium"  verification_method: "test"
  }
  requirement "REQ-CHG-005" {
    description: "AC/DC conversion efficiency shall be at least 97 % between 50 % and 100 % of rated power"
    category: "performance"  priority: "Medium"  verification_method: "test"
  }
  requirement "REQ-CHG-006" {
    description: "Energy shall be metered on the DC output by a MID-certified meter (class 1) and reported to the back end at least every 60 s during a session"
    category: "functional"  priority: "High"  source: "MID 2014/32/EU, OCPP 2.0.1 MeterValues"  verification_method: "test"
  }
  requirement "REQ-CHG-007" {
    description: "The charger shall talk to the CSMS over OCPP 2.0.1 on WebSocket with TLS 1.3 and mutual certificate authentication (security profile 3)"
    category: "interface"  priority: "High"  source: "OCPP 2.0.1 part 2"  verification_method: "test"
  }
  requirement "REQ-CHG-008" {
    description: "The site controller shall hold the site at or below 600 kW of the 630 kVA grid connection and redistribute the budget across the eight chargers within 5 s of a change"
    category: "functional"  priority: "High"  source: "Grid connection agreement"  verification_method: "test"
  }
  requirement "REQ-OPS-001" {
    description: "Every fault shall be recorded with a timestamp, the charger state and the last 10 s of measurements, readable remotely by the maintenance technician"
    category: "operations"  priority: "Medium"  verification_method: "demonstration"
  }
}

requirements safety {
  requirement "REQ-SAF-001" {
    description: "An emergency stop or a protection trip shall open the DC contactors within 100 ms and discharge the output below 60 V within 1 s"
    category: "safety"  priority: "Critical"  safety_level: "SIL-2"  source: "IEC 61851-23 clause 6, IEC 61508"  verification_method: "test"
  }
  requirement "REQ-SAF-002" {
    description: "Insulation of the DC output shall be monitored before energizing and during charging; below 100 ohm per volt the session shall stop"
    category: "safety"  priority: "Critical"  safety_level: "SIL-2"  source: "IEC 61851-23, IEC 61557-8"  verification_method: "test"
  }
  requirement "REQ-SAF-003" {
    description: "Connector and cable temperature shall be measured; current shall be derated above 70 degrees Celsius and the session stopped above 90 degrees Celsius or on loss of coolant flow"
    category: "safety"  priority: "Critical"  safety_level: "SIL-2"  source: "IEC 62196-3, cable data sheet"  verification_method: "test"
  }
  requirement "REQ-SAF-004" {
    description: "The connector shall be locked before the output is energized and shall unlock only below 60 V"
    category: "safety"  priority: "High"  safety_level: "SIL-2"  source: "IEC 61851-23 clause 6.3.1.106"  verification_method: "test"
  }
}

requirements security {
  requirement "REQ-SEC-001" {
    description: "Card data shall be handled by the certified payment terminal only; the charger shall never store or log a primary account number"
    category: "security"  priority: "Critical"  source: "PCI DSS 4.0"  verification_method: "inspection"
  }
  requirement "REQ-SEC-002" {
    description: "Firmware shall be signed by the manufacturer; the signature shall be verified at boot and before installing an image received by OCPP UpdateFirmware"
    category: "security"  priority: "High"  source: "OCPP 2.0.1 security whitepaper"  verification_method: "test"
  }
  requirement "REQ-SEC-003" {
    description: "Meter values and session records shall be signed by the meter so that a tampered record is detected by the billing service and by the driver"
    category: "security"  priority: "High"  source: "German Eichrecht, OCPP SignedMeterValue"  verification_method: "test"
  }
}

// ---- Operational analysis -------------------------------------------------

operational_analysis "Operational Analysis" {
  actor "EV driver" { id: "OA-DRV" description: "Plugs in, pays, waits twenty minutes" }
  actor "Fleet manager" { id: "OA-FLEET" description: "Books capacity, receives one invoice for the fleet" }
  actor "Site operator" { id: "OA-SITE" description: "Owns the parking site and the grid connection" }
  actor "Grid operator" { id: "OA-GRID" description: "Distribution system operator; may cap the site power" }
  actor "Maintenance technician" { id: "OA-TECH" description: "Diagnoses and repairs the chargers" }

  entity "Electric vehicle" {
    id: "OE-EV"
    activity "Request charging power" { id: "OAC-REQ" description: "The battery management system asks for the current it accepts" }
  }
  entity "Charge point operator" {
    id: "OE-CPO"
    activity "Authorize the session" { id: "OAC-AUTH" }
    activity "Deliver energy" { id: "OAC-DELIVER" }
    activity "Bill the session" { id: "OAC-BILL" }
    activity "Balance site load" { id: "OAC-BALANCE" }
    activity "Maintain the chargers" { id: "OAC-MAINTAIN" }
  }
  entity "Payment provider" {
    id: "OE-PSP"
    activity "Authorize payment" { id: "OAC-PAY" }
  }
  operational_activity "Plug in and charge" { id: "OAC-PLUG" description: "The driver parks, plugs the CCS2 connector and walks away" }
  operational_activity "Cap the site power" { id: "OAC-CAP" description: "The grid operator lowers the site limit during a congestion" }

  operational_capability "Charge a vehicle in 20 minutes" {
    id: "OC-001"
    description: "10 % to 80 % of a 77 kWh battery in twenty minutes"
    involves: ["OA-DRV", "OE-EV", "OE-CPO", "OAC-PLUG", "OAC-REQ", "OAC-DELIVER", "OP-CHARGE"]
  }
  operational_capability "Pay without an account" {
    id: "OC-002"
    involves: ["OA-DRV", "OE-PSP", "OAC-PAY", "OAC-BILL"]
    extends: ["OC-001"]
  }
  operational_capability "Balance site load against the grid" {
    id: "OC-003"
    involves: ["OA-SITE", "OA-GRID", "OE-CPO", "OAC-CAP", "OAC-BALANCE"]
  }
  operational_capability "Keep the site available" {
    id: "OC-004"
    involves: ["OA-TECH", "OA-FLEET", "OAC-MAINTAIN"]
  }

  interaction "Plug-in" { id: "OI-PLUG" from: "OA-DRV" to: "OAC-PLUG" }
  interaction "Charge request" { id: "OI-REQ" from: "OAC-REQ" to: "OAC-DELIVER" }
  interaction "Payment authorization" { id: "OI-PAY" from: "OAC-PAY" to: "OAC-AUTH" }
  interaction "Power cap" { id: "OI-CAP" from: "OA-GRID" to: "OAC-CAP" }
  interaction "Fault report" { id: "OI-FAULT" from: "OAC-MAINTAIN" to: "OA-TECH" }

  operational_process "Charge and pay" {
    id: "OP-CHARGE"
    involves: ["OAC-PLUG", "OAC-AUTH", "OAC-PAY", "OAC-REQ", "OAC-DELIVER", "OAC-BILL"]
  }
}

// ---- System analysis ------------------------------------------------------

system_analysis "System Analysis" {
  actor "Electric vehicle" { id: "SA-EV" }
  actor "EV driver" { id: "SA-DRV" }
  actor "Payment provider" { id: "SA-PSP" }
  actor "Grid operator" { id: "SA-GRID" }
  actor "Maintenance technician" { id: "SA-TECH" }

  mission "Public fast charging" {
    id: "MIS-001"
    description: "Sell energy to any electric vehicle at highway speed, around the clock"
  }
  capability "Fast charging session" {
    id: "CAP-001"  mission: "MIS-001"  realizes: "OC-001"
    involves: ["SF-AUTH", "SF-NEGO", "SF-CONV", "SF-REG", "SF-METER", "FC-START"]
  }
  capability "Ad-hoc payment" {
    id: "CAP-002"  mission: "MIS-001"  realizes: "OC-002"
    involves: ["SF-AUTH", "SF-METER", "SF-REPORT", "SA-PSP"]
    extends: ["CAP-001"]
  }
  capability "Site load management" {
    id: "CAP-003"  mission: "MIS-001"  realizes: "OC-003"
    involves: ["SF-BAL", "SF-REG", "SA-GRID"]
  }
  capability "Safe shutdown" {
    id: "CAP-004"  mission: "MIS-001"  realizes: "OC-004"
    involves: ["SF-ESTOP", "SF-MON", "SF-STOP", "SF-REPORT", "FC-ESTOP", "FC-TRIP"]
  }

  function "Authenticate session" {
    id: "SF-AUTH"
    description: "Contract certificate (ISO 15118-2), RFID token or payment pre-authorization"
    latency: 1500 ms
    port in "credential"
    port out "authorization"
  }
  function "Negotiate charging parameters with the vehicle" {
    id: "SF-NEGO"
    description: "ChargeParameterDiscovery and PowerDelivery: voltage, current and energy limits of the EV"
    latency: 3000 ms
    port in "authorization"
    port in "ev_request"
    port out "schedule"
  }
  function "Regulate current" {
    id: "SF-REG"
    description: "Follows the EV current request within the site power budget"
    latency: 500 ms
    period: 100 ms
    port in "schedule"
    port in "power_limit"
    port out "current_command"
  }
  function "Convert AC to DC" {
    id: "SF-CONV"
    description: "Pre-charges to the battery voltage, closes the contactors, ramps the current"
    latency: 2000 ms
    port in "current_command"
    port out "dc_measurement"
    port out "output_power"
  }
  function "Monitor insulation and temperature" {
    id: "SF-MON"
    is: "Safety function"
    description: "Insulation monitoring device on the DC output, connector and cable temperatures, coolant flow"
    latency: 20 ms
    period: 10 ms
    port in "dc_measurement"
    port out "fault"
  }
  function "Detect emergency stop" {
    id: "SF-ESTOP"
    is: "Safety function"
    description: "Reads the mushroom button through a dual-channel safety input"
    latency: 10 ms
    port in "estop_button"
    port out "stop_request"
  }
  function "Stop on fault" {
    id: "SF-STOP"
    is: "Safety function"
    description: "Opens the DC contactors, inhibits the power modules, discharges the output"
    latency: 40 ms
    port in "fault"
    port in "stop_request"
    port out "fault_record"
  }
  function "Meter energy" {
    id: "SF-METER"
    description: "DC energy meter, signed meter values"
    latency: 100 ms
    period: 1 s
    port in "output_power"
    port out "meter_values"
  }
  function "Report to the back end" {
    id: "SF-REPORT"
    description: "OCPP 2.0.1 TransactionEvent, MeterValues, StatusNotification, NotifyEvent"
    latency: 200 ms
    port in "meter_values"
    port in "fault_record"
    port out "site_status"
  }
  function "Balance load across chargers" {
    id: "SF-BAL"
    description: "Shares the 600 kW site budget between the eight chargers on demand and grid signals"
    latency: 1000 ms
    period: 1 s
    port in "site_status"
    port in "grid_signal"
    port out "power_limit"
  }

  functional_exchange "session authorization" { id: "FE-AUTH" from: "SF-AUTH.authorization" to: "SF-NEGO.authorization" }
  functional_exchange "charging schedule" { id: "FE-SCHED" from: "SF-NEGO.schedule" to: "SF-REG.schedule" }
  functional_exchange "current command" { id: "FE-CMD" from: "SF-REG.current_command" to: "SF-CONV.current_command" }
  functional_exchange "DC measurement" { id: "FE-DC" from: "SF-CONV.dc_measurement" to: "SF-MON.dc_measurement" }
  functional_exchange "protection fault" { id: "FE-FAULT" from: "SF-MON.fault" to: "SF-STOP.fault" }
  functional_exchange "stop request" { id: "FE-STOPREQ" from: "SF-ESTOP.stop_request" to: "SF-STOP.stop_request" }
  functional_exchange "output power" { id: "FE-POWER" from: "SF-CONV.output_power" to: "SF-METER.output_power" }
  functional_exchange "meter values" { id: "FE-METER" from: "SF-METER.meter_values" to: "SF-REPORT.meter_values" }
  functional_exchange "fault record" { id: "FE-RECORD" from: "SF-STOP.fault_record" to: "SF-REPORT.fault_record" }
  functional_exchange "site status" { id: "FE-STATUS" from: "SF-REPORT.site_status" to: "SF-BAL.site_status" }
  functional_exchange "power limit" { id: "FE-LIMIT" from: "SF-BAL.power_limit" to: "SF-REG.power_limit" }

  functional_chain "Emergency stop" {
    id: "FC-ESTOP"
    description: "Button pressed to contactors open: the 100 ms of REQ-SAF-001"
    capability: "CAP-004"
    involves: ["SF-ESTOP", "SF-STOP"]
    latency_budget: 100 ms
  }
  functional_chain "Protection trip" {
    id: "FC-TRIP"
    description: "Insulation or temperature fault to contactors open"
    capability: "CAP-004"
    involves: ["SF-MON", "SF-STOP"]
    latency_budget: 100 ms
  }
  functional_chain "Session start" {
    id: "FC-START"
    description: "Authorization to energy flowing: the 10 s of REQ-CHG-004"
    capability: "CAP-001"
    involves: ["SF-AUTH", "SF-NEGO", "SF-REG", "SF-CONV"]
    latency_budget: 10 s
  }
}

// ---- Logical architecture -------------------------------------------------

logical_architecture "Logical Architecture" {
  component "Charge controller" {
    id: "LC-CTL"
    description: "Session state machine, current regulation, protection logic, secure boot"
    safety_level: "SIL-2"
    port in "schedule"  port in "protection_status"  port in "power_limit"  port in "payment"  port in "coolant_status"
    port out "setpoint"  port out "session_status"  port out "transaction_events"
    function "Regulate charging current" { id: "LF-REG" realizes: "SF-REG" }
    function "Stop on fault" { id: "LF-STOP" is: "Safety function" latency: 40 ms realizes: "SF-STOP" }
    function "Run the session state machine" { id: "LF-SESSION" description: "Available, Preparing, Charging, SuspendedEV, Finishing, Faulted, Unavailable" }
    function "Verify firmware signature" { id: "LF-BOOT" description: "Secure boot; signature check of any image received by UpdateFirmware" }
  }
  component "Power conversion modules" {
    id: "LC-PWR"
    is: "DC power module"
    description: "Seven 50 kW modules in parallel per connector"
    multiplicity: 7
    port in "setpoint"  port in "trip"
    port out "dc_measurement"
    function "Convert AC to DC" { id: "LF-CONV" realizes: "SF-CONV" }
  }
  component "Protection and metering" {
    id: "LC-PROT"
    description: "Insulation monitoring device, temperature inputs, dual-channel emergency stop, DC energy meter"
    safety_level: "SIL-2"
    port in "dc_measurement"  port in "estop"
    port out "protection_status"  port out "trip"  port out "meter_values"
    function "Monitor insulation and temperature" { id: "LF-IMD" is: "Safety function" latency: 20 ms realizes: "SF-MON" }
    function "Detect emergency stop" { id: "LF-ESTOP" is: "Safety function" latency: 10 ms realizes: "SF-ESTOP" }
    function "Meter and sign energy" { id: "LF-METER" realizes: "SF-METER" }
  }
  component "Cable and connector cooling" {
    id: "LC-COOL"
    description: "Coolant pump, heat exchanger and the temperature sensors in the CCS2 connector"
    port out "coolant_status"
    function "Control coolant flow on connector temperature" { id: "LF-COOL" }
  }
  component "HMI and payment" {
    id: "LC-HMI"
    description: "Screen, RFID reader and the certified EMV payment terminal"
    port in "session_status"
    port out "payment"
    function "Guide the driver" { id: "LF-UI" }
    function "Authorize by RFID or payment card" { id: "LF-PAY" realizes: "SF-AUTH" }
  }
  component "Communications" {
    id: "LC-COM"
    description: "The two protocol stacks of the charger"
    component "OCPP client" {
      id: "LC-OCPP"
      port in "transaction_events"
      port out "ocpp"
      function "Report to the CSMS" { id: "LF-OCPP" realizes: "SF-REPORT" }
    }
    component "ISO 15118 stack" {
      id: "LC-15118"
      port in "v2g_request"
      port out "v2g_response"  port out "schedule"
      function "Authenticate by contract certificate" { id: "LF-PNC" realizes: "SF-AUTH" }
      function "Negotiate charging parameters" { id: "LF-NEGO" realizes: "SF-NEGO" }
    }
  }
  component "Site controller" {
    id: "LC-SITE"
    description: "Load management for the eight chargers of the site"
    site_power_cap: 600 kW
    port in "grid_signal"  port in "charger_status"
    port out "power_limit"  port out "site_telemetry"
    function "Balance load across chargers" { id: "LF-BAL" realizes: "SF-BAL" }
  }
  component "Back-end services" {
    id: "LC-BACKEND"
    description: "The charge point operator platform"
    component "CSMS" {
      id: "LC-CSMS"
      port in "ocpp"
      port out "session_records"
      function "Manage charge points" { id: "LF-CSMS" description: "OCPP 2.0.1 server: authorization, transactions, firmware, charging profiles" realizes: "SF-REPORT" }
    }
    component "Billing" {
      id: "LC-BILL"
      port in "session_records"
      function "Bill the session" { id: "LF-BILL" }
    }
    component "Monitoring" {
      id: "LC-MON"
      port in "site_telemetry"
      function "Monitor fleet health" { id: "LF-MON" description: "Availability, fault records, remote diagnostics" }
    }
  }
  component "Electric vehicle" {
    id: "LC-EV"
    description: "External: battery management system and ISO 15118 EV communication controller"
    port in "v2g_response"
    port out "v2g_request"
    function "Control the battery charging" { id: "LF-EV" }
  }

  component_exchange "V2G request" { id: "CE-V2G-REQ" from_port: "LC-EV.v2g_request" to_port: "LC-15118.v2g_request" protocol: "ISO 15118-2 over HomePlug Green PHY" }
  component_exchange "V2G response" { id: "CE-V2G-RSP" from_port: "LC-15118.v2g_response" to_port: "LC-EV.v2g_response" protocol: "ISO 15118-2 over HomePlug Green PHY" }
  component_exchange "Charging schedule" { id: "CE-SCHED" from_port: "LC-15118.schedule" to_port: "LC-CTL.schedule" }
  component_exchange "Payment authorization" { id: "CE-PAY" from_port: "LC-HMI.payment" to_port: "LC-CTL.payment" }
  component_exchange "Session status" { id: "CE-STATUS" from_port: "LC-CTL.session_status" to_port: "LC-HMI.session_status" }
  component_exchange "Module setpoint" { id: "CE-SETPOINT" from_port: "LC-CTL.setpoint" to_port: "LC-PWR.setpoint" protocol: "CAN FD" period: 10 ms }
  component_exchange "DC measurement" { id: "CE-DC" from_port: "LC-PWR.dc_measurement" to_port: "LC-PROT.dc_measurement" }
  component_exchange "Protection status" { id: "CE-PROT" from_port: "LC-PROT.protection_status" to_port: "LC-CTL.protection_status" }
  component_exchange "Trip" { id: "CE-TRIP" from_port: "LC-PROT.trip" to_port: "LC-PWR.trip" description: "Hard-wired inhibit, independent of the controller" }
  component_exchange "Coolant status" { id: "CE-COOL" from_port: "LC-COOL.coolant_status" to_port: "LC-CTL.coolant_status" }
  component_exchange "Transaction events" { id: "CE-TXN" from_port: "LC-CTL.transaction_events" to_port: "LC-OCPP.transaction_events" }
  component_exchange "OCPP messages" { id: "CE-OCPP" from_port: "LC-OCPP.ocpp" to_port: "LC-CSMS.ocpp" protocol: "OCPP 2.0.1 / WebSocket / TLS 1.3" }
  component_exchange "Session records" { id: "CE-RECORDS" from_port: "LC-CSMS.session_records" to_port: "LC-BILL.session_records" }
  component_exchange "Power limit" { id: "CE-LIMIT" from_port: "LC-SITE.power_limit" to_port: "LC-CTL.power_limit" protocol: "OCPP 2.0.1 SetChargingProfile" }
  component_exchange "Site telemetry" { id: "CE-TELEMETRY" from_port: "LC-SITE.site_telemetry" to_port: "LC-MON.site_telemetry" }

  interface "OCPP 2.0.1" { id: "IF-OCPP" from: "LC-OCPP" to: "LC-CSMS" description: "OCPP 2.0.1-J, WebSocket, TLS 1.3, security profile 3" }
  interface "ISO 15118-2" { id: "IF-15118" from: "LC-15118" to: "LC-EV" description: "V2GTP/EXI over ISO 15118-3 power-line communication" }
  interface "Power module bus" { id: "IF-CAN" from: "LC-CTL" to: "LC-PWR" description: "CAN FD at 2 Mbps" }

  capability_realization "Fast charging session" { id: "CR-001" realizes: "CAP-001" involves: ["LC-15118", "LC-CTL", "LC-PWR", "LC-PROT", "LC-OCPP"] }
  capability_realization "Safe shutdown" { id: "CR-004" realizes: "CAP-004" involves: ["LC-PROT", "LC-CTL", "LC-PWR"] }
}

// ---- Physical architecture ------------------------------------------------

physical_architecture "Physical Architecture" {
  node "Charger cabinet" {
    id: "PN-CHG"
    description: "Control cabinet of one charger: controller board, protection, cellular modem"
    safety_level: "SIL-2"
    processor: "Arm Cortex-A53; lockstep Cortex-M4 safety MCU"
    ram: 1 GB  flash: 8 GB  power: 150 W
    port out "can"  port out "wan"  port in "lan"
    deploys "LC-CTL"
    deploys "LC-PROT"
    deploys "LC-OCPP"
    behavior_component "Charge controller firmware" {
      id: "BC-CTL"
      allocated_component: "LC-CTL"
      safety_level: "SIL-2"
      function "Regulate current at 100 Hz" { id: "PF-REG" realizes: "LF-REG" period: 10 ms }
      function "Open contactors on trip" { id: "PF-STOP" realizes: "LF-STOP" latency: 40 ms }
    }
    hardware_component "Insulation monitoring device" { id: "HW-IMD" description: "IEC 61557-8 IMD, 100 ohm per volt threshold" voltage: 1000 V power: 5 W }
    hardware_component "DC contactors" { id: "HW-CONTACTOR" description: "Two 500 A contactors with mirror contacts in series with the output" voltage: 1000 V mass: 4 kg }
  }
  node "Power cabinet" {
    id: "PN-PWR"
    description: "Seven 50 kW power modules, AC input filter, DC output bus"
    power: 350 kW  voltage: 1000 V  rated_current: 500 A  mass: 800 kg
    port in "can"
    deploys "LC-PWR"
    hardware_component "Power module" { id: "HW-MODULE" description: "Isolated 50 kW AC/DC brick, 97 % peak efficiency" power: 50 kW voltage: 1000 V mass: 28 kg }
  }
  node "Dispenser" {
    id: "PN-DISP"
    description: "User unit at the parking bay: screen, payment terminal, CCS2 cable and its cooling"
    mass: 350 kg
    port in "cp"
    deploys "LC-HMI"
    deploys "LC-COOL"
    deploys "LC-15118"
    hardware_component "Liquid-cooled CCS2 cable" { id: "HW-CABLE" description: "4 m cable, 500 A continuous with coolant" voltage: 1000 V mass: 9 kg }
    hardware_component "Payment terminal" { id: "HW-PAYTERM" description: "PCI PTS 6 certified EMV contactless terminal" power: 8 W }
    hardware_component "Coolant pump" { id: "HW-PUMP" power: 60 W }
  }
  node "Site controller box" {
    id: "PN-SITE"
    description: "Industrial PC and router in the site switchboard room"
    ram: 4 GB  flash: 64 GB  power: 40 W
    port out "lan"
    deploys "LC-SITE"
  }
  node "Cloud back end" {
    id: "PN-CLOUD"
    description: "CPO platform on two cloud regions"
    redundancy: "active-active, two regions"
    port in "ocpp"
    deploys "LC-CSMS"
    deploys "LC-BILL"
    deploys "LC-MON"
  }
  node "Grid connection" {
    id: "PN-GRID"
    description: "20 kV / 400 V transformer and low-voltage switchgear, 630 kVA"
    voltage: 400 V  power: 630 kW
    hardware_component "630 kVA transformer" { id: "HW-TRAFO" voltage: 400 V mass: 1900 kg }
  }
  node "Electric vehicle" {
    id: "PN-EV"
    description: "External: CCS2 inlet, EV communication controller, battery"
    port out "inlet"
    deploys "LC-EV"
  }

  physical_link "Power module CAN" { id: "PL-CAN" from: "PN-CHG" to: "PN-PWR" protocol: "CAN FD" bandwidth: 2 Mbps }
  physical_link "Dispenser Ethernet" { id: "PL-ETH-DISP" from: "PN-CHG" to: "PN-DISP" protocol: "Ethernet 100BASE-TX" bandwidth: 100 Mbps }
  physical_link "Control pilot PLC" { id: "PL-PLC" from: "PN-DISP" to: "PN-EV" protocol: "HomePlug Green PHY (ISO 15118-3) on the control pilot" bandwidth: 10 Mbps }
  physical_link "CCS2 DC cable" { id: "PL-CABLE" from: "PN-DISP" to: "PN-EV" protocol: "DC 1000 V / 500 A, liquid-cooled" }
  physical_link "DC power bus" { id: "PL-DCBUS" from: "PN-PWR" to: "PN-DISP" protocol: "DC bus 1000 V, 2 x 240 mm2" }
  physical_link "AC feed" { id: "PL-AC" from: "PN-GRID" to: "PN-PWR" protocol: "AC 400 V three-phase, 50 Hz" }
  physical_link "Site Ethernet" { id: "PL-ETH-SITE" from: "PN-SITE" to: "PN-CHG" protocol: "Ethernet 1000BASE-T" bandwidth: 1 Gbps }
  physical_link "Cellular uplink" { id: "PL-LTE" from: "PN-CHG" to: "PN-CLOUD" protocol: "LTE Cat 4, private APN" bandwidth: 50 Mbps }
  physical_link "Site WAN" { id: "PL-WAN" from: "PN-SITE" to: "PN-CLOUD" protocol: "Fibre, IPsec tunnel" bandwidth: 100 Mbps }

  physical_exchange "Module setpoint frame" {
    id: "PX-SETPOINT"  from: "PN-CHG.can"  to: "PN-PWR.can"  via: "PL-CAN"
    message_type: "CAN FD frame 0x120"  frequency: "100 Hz"  size: 64 B
  }
  physical_exchange "OCPP meter values" {
    id: "PX-METER"  from: "PN-CHG.wan"  to: "PN-CLOUD.ocpp"  via: "PL-LTE"
    message_type: "OCPP TransactionEvent with MeterValues (JSON)"  frequency: "0.017 Hz"  size: 2 kB
  }
  physical_exchange "Charging profile" {
    id: "PX-PROFILE"  from: "PN-SITE.lan"  to: "PN-CHG.lan"  via: "PL-ETH-SITE"
    message_type: "OCPP SetChargingProfile (JSON)"  frequency: "1 Hz"  size: 1 kB
  }
  physical_exchange "V2G session" {
    id: "PX-V2G"  from: "PN-EV.inlet"  to: "PN-DISP.cp"  via: "PL-PLC"
    message_type: "ISO 15118-2 V2GTP / EXI"  frequency: "1 Hz"  size: 4 kB
  }

  physical_path "OCPP route" { id: "PP-OCPP" involves: ["Cellular uplink"] }
  physical_path "Load management route" { id: "PP-LOAD" involves: ["Site WAN", "Site Ethernet"] }
}

// ---- EPBS: configuration items --------------------------------------------

epbs "Charging station EPBS" {
  system "DC fast-charging station" {
    id: "CI-STATION"
    subsystem "Charger" {
      id: "CI-CHARGER"
      item "Charger cabinet" { id: "CI-CHG" part_number: "CHG-350-CAB-02" }
      item "Power cabinet" { id: "CI-PWR" part_number: "PWR-350-7M-01" mass: 800 kg }
      item "Dispenser with liquid-cooled CCS2 cable" { id: "CI-DISP" part_number: "DSP-CCS2-LC-04" mass: 350 kg }
      item "Charger firmware" { id: "CI-FW" part_number: "FW-CHG-4.3.0" }
    }
    subsystem "Site" {
      id: "CI-SITE"
      item "Site controller box" { id: "CI-SITEBOX" part_number: "SITE-LM-01" }
      item "Grid connection" { id: "CI-GRID" part_number: "GRID-630-TR" supplier: "Distribution system operator" }
    }
    subsystem "Back end" {
      id: "CI-BACKEND"
      item "CSMS release" { id: "CI-CSMS" part_number: "CSMS-2026.10" }
    }
  }
}

// ---- Data model -----------------------------------------------------------

data_type "Energy" { id: "DT-ENERGY" base: "Real" unit: "kWh" }
data_type "Current" { id: "DT-CURRENT" base: "Real" unit: "A" }
data_type "Voltage" { id: "DT-VOLTAGE" base: "Real" unit: "V" }
data_type "Temperature" { id: "DT-TEMP" base: "Real" unit: "K" }
data_type "Timestamp" { id: "DT-TIME" base: "String" }

enumeration "ConnectorType" { id: "EN-CONNECTOR" values: ["CCS2", "CHAdeMO", "NACS", "Type2"] }
enumeration "SessionState" { id: "EN-SESSION" values: ["Available", "Preparing", "Charging", "SuspendedEV", "Finishing", "Faulted", "Unavailable"] }
enumeration "AuthorizationMethod" { id: "EN-AUTH" values: ["PlugAndCharge", "RFID", "PaymentCard", "RemoteStart"] }

class "VehicleIdentity" {
  id: "CL-VEHICLE"
  description: "What ISO 15118 tells the charger about the vehicle"
  evcc_id: "String"  contract_certificate_id: "String"  battery_capacity: "Energy"
}
class "Tariff" {
  id: "CL-TARIFF"
  tariff_id: "String"  price_per_kwh: "Real"  price_per_minute: "Real"  currency: "String"
}
class "MeterValue" {
  id: "CL-METER"
  timestamp: "Timestamp"  energy: "Energy"  voltage: "Voltage"  current: "Current"  signature: "String"
}
class "ChargingSession" {
  id: "CL-SESSION"
  session_id: "String"  connector: "ConnectorType"  state: "SessionState"  authorization: "AuthorizationMethod"
  vehicle: "VehicleIdentity"  tariff: "Tariff"  meter_values: "MeterValue"  energy_delivered: "Energy"
}
class "FaultRecord" {
  id: "CL-FAULT"
  timestamp: "Timestamp"  code: "String"  session_state: "SessionState"  connector_temperature: "Temperature"  insulation_resistance: "Real"
}

association "session identifies vehicle" { id: "AS-VEHICLE" from: "CL-SESSION" to: "CL-VEHICLE" role: "vehicle" kind: "association" multiplicity: "1" }
association "session priced by tariff" { id: "AS-TARIFF" from: "CL-SESSION" to: "CL-TARIFF" role: "tariff" kind: "association" multiplicity: "1" }
association "session holds meter values" { id: "AS-METER" from: "CL-SESSION" to: "CL-METER" role: "meter_values" kind: "composition" multiplicity: "1..*" }

exchange_item "SessionAuthorization" { id: "EI-AUTH" mechanism: "OPERATION" elements: ["CL-VEHICLE"] }
exchange_item "ChargingSchedule" { id: "EI-SCHED" mechanism: "DATA" }
exchange_item "SignedMeterValues" { id: "EI-METER" mechanism: "FLOW" elements: ["CL-METER"] }
exchange_item "TransactionEvent" { id: "EI-TXN" mechanism: "EVENT" elements: ["CL-SESSION", "CL-METER"] }
exchange_item "EmergencyStop" { id: "EI-ESTOP" mechanism: "EVENT" }
exchange_item "FaultReport" { id: "EI-FAULT" mechanism: "EVENT" elements: ["CL-FAULT"] }
exchange_item "PaymentPreAuthorization" { id: "EI-PAY" mechanism: "OPERATION" elements: ["CL-TARIFF"] }
exchange_item "FirmwareImage" { id: "EI-FW" mechanism: "SHARED_DATA" }

// ---- Modes and states -----------------------------------------------------
// OCPP connector states of one charger, then the power mode of the site.
// Transitions are triggered by the system functions declared above.

state_machine "Charger states" {
  id: "SM-CHARGER"
  initial: "Available"
  state Available { description: "Connector free, no fault" }
  state Preparing { description: "Plugged in, authorizing and negotiating" }
  state Charging { description: "Contactors closed, energy flowing" }
  state SuspendedEV { description: "The vehicle paused the charge (battery balancing, temperature)" }
  state Finishing { description: "Contactors open, output discharging, connector still locked" }
  state Faulted { description: "Protection tripped; output isolated" }
  state Unavailable { description: "Out of service by the operator or awaiting repair" }

  transition Available -> Preparing { trigger: "SF-AUTH" guard: "connector plugged and locked" }
  transition Preparing -> Charging { trigger: "SF-CONV" guard: "cable check passed, insulation above 100 ohm/V" }
  transition Preparing -> Available { trigger: "SF-AUTH" guard: "authorization refused or unplugged" }
  transition Charging -> SuspendedEV { trigger: "SF-NEGO" guard: "EV requests 0 A" }
  transition SuspendedEV -> Charging { trigger: "SF-NEGO" guard: "EV requests current again" }
  transition Charging -> Finishing { trigger: "SF-NEGO" guard: "target energy reached or stop requested" }
  transition SuspendedEV -> Finishing { trigger: "SF-NEGO" guard: "session ended by EV or driver" }
  transition Finishing -> Available { trigger: "SF-METER" guard: "output below 60 V, connector unlocked and removed" }
  transition Preparing -> Faulted { trigger: "SF-STOP" timing: 100 ms }
  transition Charging -> Faulted { trigger: "SF-STOP" timing: 100 ms action: "open contactors, inhibit modules, discharge output" }
  transition SuspendedEV -> Faulted { trigger: "SF-STOP" timing: 100 ms }
  transition Faulted -> Available { trigger: "SF-REPORT" guard: "fault self-cleared and output insulated" }
  transition Faulted -> Unavailable { trigger: "SF-REPORT" guard: "fault not self-clearing" }
  transition Available -> Unavailable { trigger: "SF-REPORT" guard: "ChangeAvailability inoperative" }
  transition Unavailable -> Available { trigger: "SF-REPORT" guard: "technician reset or ChangeAvailability operative" }
}

state_machine "Site power modes" {
  id: "SM-SITE"
  initial: "Full power"
  mode "Full power" { description: "600 kW budget shared on demand" }
  mode "Derated" { description: "Grid operator cap applied; chargers share the reduced budget" }
  mode "Islanded" { description: "Back-end link lost; chargers hold their last profile" }
  transition "Full power" -> "Derated" { trigger: "SF-BAL" guard: "grid signal below site demand" }
  transition "Derated" -> "Full power" { trigger: "SF-BAL" guard: "grid signal released" }
  transition "Full power" -> "Islanded" { trigger: "SF-REPORT" guard: "no CSMS heartbeat for 3 minutes" }
  transition "Islanded" -> "Full power" { trigger: "SF-REPORT" guard: "CSMS reachable" }
}

// ---- Scenarios ------------------------------------------------------------

scenario "Plug-and-charge session" {
  id: "SC-PNC"
  owner: "CAP-001"
  participants: ["LC-EV", "LC-15118", "LC-CTL", "LC-PWR", "LC-PROT", "LC-OCPP", "LC-CSMS"]
  state "LC-CTL" "Preparing"
  message "LC-EV" -> "LC-15118" "SessionSetup, PaymentDetails (contract certificate)" { exchange: "V2G request" type: "sync" }
  message "LC-15118" -> "LC-EV" "AuthorizationRes OK" { exchange: "V2G response" type: "sync" timing: 1500 ms }
  message "LC-EV" -> "LC-15118" "ChargeParameterDiscovery: 400 V, 500 A max, 60 kWh" { exchange: "V2G request" type: "sync" }
  message "LC-15118" -> "LC-CTL" "charging schedule" { exchange: "Charging schedule" type: "async" }
  message "LC-CTL" -> "LC-OCPP" "TransactionEvent Started" { exchange: "Transaction events" type: "async" }
  message "LC-OCPP" -> "LC-CSMS" "TransactionEvent Started" { exchange: "OCPP messages" type: "async" }
  message "LC-PROT" -> "LC-CTL" "insulation 850 ohm/V, cable check passed" { exchange: "Protection status" type: "async" }
  message "LC-CTL" -> "LC-PWR" "pre-charge to 400 V, close contactors" { exchange: "Module setpoint" type: "async" timing: 2000 ms }
  state "LC-CTL" "Charging"
  fragment loop "until target energy or EV stop" {
    message "LC-EV" -> "LC-15118" "CurrentDemand 480 A" { exchange: "V2G request" type: "sync" }
    message "LC-15118" -> "LC-CTL" "current request" { exchange: "Charging schedule" type: "async" }
    message "LC-CTL" -> "LC-PWR" "setpoint 480 A" { exchange: "Module setpoint" type: "async" timing: 100 ms }
    message "LC-PWR" -> "LC-PROT" "DC 402 V, 480 A" { exchange: "DC measurement" type: "async" }
    fragment opt "every 60 s" {
      message "LC-CTL" -> "LC-OCPP" "TransactionEvent Updated, signed meter values" { exchange: "Transaction events" type: "async" }
      message "LC-OCPP" -> "LC-CSMS" "TransactionEvent Updated" { exchange: "OCPP messages" type: "async" }
    }
  }
  state "LC-CTL" "Finishing"
  message "LC-EV" -> "LC-15118" "PowerDelivery Stop, SessionStop" { exchange: "V2G request" type: "sync" }
  message "LC-15118" -> "LC-CTL" "session stop" { exchange: "Charging schedule" type: "async" }
  message "LC-CTL" -> "LC-PWR" "open contactors, discharge" { exchange: "Module setpoint" type: "async" timing: 1000 ms }
  message "LC-CTL" -> "LC-OCPP" "TransactionEvent Ended, 54.2 kWh" { exchange: "Transaction events" type: "async" }
  message "LC-OCPP" -> "LC-CSMS" "TransactionEvent Ended" { exchange: "OCPP messages" type: "async" }
}

scenario "Emergency stop" {
  id: "SC-ESTOP"
  owner: "CAP-004"
  participants: ["SA-DRV", "LC-PROT", "LC-PWR", "LC-CTL", "LC-OCPP", "LC-CSMS"]
  message "SA-DRV" -> "LC-PROT" "emergency stop button pressed" { type: "async" }
  message "LC-PROT" -> "LC-PWR" "trip: inhibit modules, open contactors" { exchange: "Trip" type: "async" timing: 50 ms }
  message "LC-PROT" -> "LC-CTL" "protection status: emergency stop" { exchange: "Protection status" type: "async" }
  state "LC-CTL" "Faulted"
  message "LC-CTL" -> "LC-OCPP" "StatusNotification Faulted, NotifyEvent EmergencyStop" { exchange: "Transaction events" type: "async" }
  message "LC-OCPP" -> "LC-CSMS" "StatusNotification Faulted" { exchange: "OCPP messages" type: "async" }
}

// ---- Safety analysis ------------------------------------------------------
// IEC 61508 context; the hazards are classified with the S/E/C table the
// compiler checks, each mitigated by declared safety requirements.

safety_analysis {
  hazard "Electric shock at the DC output" {
    id: "HAZ-001"
    description: "Insulation fault or energized connector accessible to the driver at up to 1000 V"
    severity: "S3"  exposure: "E2"  controllability: "C3"  asil: "ASIL-B"
    mitigated_by: ["REQ-SAF-001", "REQ-SAF-002", "REQ-SAF-004"]
  }
  hazard "Thermal runaway of the connector or cable" {
    id: "HAZ-002"
    description: "Loss of coolant flow or degraded contact at 500 A: the connector overheats, burn and fire hazard"
    severity: "S2"  exposure: "E3"  controllability: "C3"  asil: "ASIL-B"
    mitigated_by: ["REQ-SAF-003", "REQ-SAF-001"]
  }
  hazard "Grid overload of the site connection" {
    id: "HAZ-003"
    description: "Eight chargers at full power exceed the 630 kVA connection: the breaker trips, site and neighboring loads lose supply"
    severity: "S1"  exposure: "E4"  controllability: "C2"  asil: "ASIL-A"
    mitigated_by: ["REQ-CHG-008"]
  }
  hazard "Unexpected energization while the connector is unlocked" {
    id: "HAZ-004"
    description: "Contactors close, or stay closed, while the connector may be withdrawn"
    severity: "S3"  exposure: "E1"  controllability: "C3"  asil: "ASIL-A"
    mitigated_by: ["REQ-SAF-004", "REQ-SAF-001"]
  }

  fmea "DC contactor welded closed" {
    id: "FMEA-001"
    failure_mode: "Contactor stuck closed after a stop command"
    effect: "Output remains energized; the connector lock cannot release"
    cause: "Contact welding on a trip under load"
    severity: "9"  occurrence: "3"  detection: "2"  rpn: 54
    element: "LC-PWR"  leads_to: ["HAZ-004", "HAZ-001"]  fit: 20
  }
  fmea "Coolant pump failure" {
    id: "FMEA-002"
    failure_mode: "Pump stops or coolant leaks"
    effect: "Cable temperature rises at 500 A; derating then stop"
    cause: "Pump wear, hose fatigue"
    severity: "7"  occurrence: "4"  detection: "2"  rpn: 56
    element: "LC-COOL"  leads_to: ["HAZ-002"]  fit: 50
  }
  fmea "Insulation monitoring device silent failure" {
    id: "FMEA-003"
    failure_mode: "IMD reports healthy insulation while its measurement is stuck"
    effect: "Insulation fault not detected during charging"
    cause: "Measurement coupling failure"
    severity: "9"  occurrence: "2"  detection: "4"  rpn: 72
    element: "LC-PROT"  leads_to: ["HAZ-001"]  fit: 15
  }
  fmea "OCPP link loss" {
    id: "FMEA-004"
    failure_mode: "Cellular uplink down for more than 3 minutes"
    effect: "New sessions authorized from the local cache only; meter values queued"
    cause: "Operator outage, antenna damage"
    severity: "4"  occurrence: "5"  detection: "1"  rpn: 20
  }
}

// ---- Cybersecurity analysis (Capella cybersecurity viewpoint) -------------

security_scales {
  id: "SEC-SCALES"
  confidentiality: ["0", "1", "2", "3"]
  integrity: ["0", "1", "2", "3"]
  availability: ["0", "1", "2", "3"]
  traceability: ["0", "1", "2", "3"]
  threat_kinds: ["Spoofing", "Tampering", "Eavesdropping", "Denial of service", "Theft"]
}

security_analysis {
  id: "SEC-LA"
  level: "la"
  asset "Payment data" {
    id: "AS-PAYMENT"  kind: "information"
    members: ["EI-PAY", "LF-PAY"]
    security_confidentiality: "3"  security_integrity: "3"
  }
  asset "Charger firmware" {
    id: "AS-FIRMWARE"  kind: "functional"
    members: ["EI-FW", "LF-BOOT"]
    security_integrity: "3"  security_availability: "2"
  }
  asset "Session records" {
    id: "AS-RECORDS"  kind: "information"
    members: ["CL-SESSION", "CL-METER", "EI-METER", "EI-TXN"]
    security_integrity: "3"  security_traceability: "3"  security_confidentiality: "2"
  }
  threat "OCPP spoofing" {
    id: "TH-OCPP"  kind: "Spoofing"  level: 3
    rationale: "A rogue CSMS or a man-in-the-middle starts free sessions, changes charging profiles or pushes firmware"
    applies_to: ["AS-RECORDS", "AS-FIRMWARE"]
    involves: ["LC-OCPP", "LC-CSMS"]
    security_integrity: "3"  security_availability: "2"
  }
  threat "Payment skimming" {
    id: "TH-SKIM"  kind: "Theft"  level: 3
    rationale: "A skimmer on the terminal or a compromised HMI captures card data"
    applies_to: ["AS-PAYMENT"]
    involves: ["LC-HMI"]
    security_confidentiality: "3"
  }
  threat "Firmware tampering" {
    id: "TH-FW"  kind: "Tampering"  level: 3
    rationale: "A modified image disables protections or alters meter values"
    applies_to: ["AS-FIRMWARE", "AS-RECORDS"]
    involves: ["LC-CTL", "LC-PROT"]
    security_integrity: "3"
  }
}

// ---- Verification ---------------------------------------------------------

test_case "TC-CHG-001" { id: "TC-CHG-001" method: "test" verifies: ["REQ-CHG-001"] description: "Rated output 1000 V / 500 A for 30 minutes on a battery emulator; cable below 70 degrees Celsius" }
test_case "TC-CHG-002" { id: "TC-CHG-002" method: "test" verifies: ["REQ-CHG-002"] description: "ISO 15118-2 conformance: PaymentDetails with contract certificate, no driver action; RFID and card fallbacks" }
test_case "TC-CHG-003" { id: "TC-CHG-003" method: "demonstration" verifies: ["REQ-CHG-003"] description: "Contactless card session: pre-authorization and settlement on the metered energy" }
test_case "TC-CHG-004" { id: "TC-CHG-004" method: "test" verifies: ["REQ-CHG-004"] description: "Authorization-to-current timing on 20 sessions with three vehicle models" }
test_case "TC-CHG-005" { id: "TC-CHG-005" method: "test" verifies: ["REQ-CHG-005"] description: "Efficiency sweep 50-100 % load with calibrated AC and DC power analyzers" }
test_case "TC-CHG-006" { id: "TC-CHG-006" method: "test" verifies: ["REQ-CHG-006"] description: "Meter accuracy against the reference meter; MeterValues interval observed on the CSMS" }
test_case "TC-CHG-007" { id: "TC-CHG-007" method: "test" verifies: ["REQ-CHG-007"] description: "OCPP 2.0.1 certification tool, security profile 3; TLS handshake inspection" }
test_case "TC-CHG-008" { id: "TC-CHG-008" method: "test" verifies: ["REQ-CHG-008"] description: "Eight chargers at full demand on the site simulator; grid cap stepped 600 kW to 300 kW, redistribution time measured" }
test_case "TC-OPS-001" { id: "TC-OPS-001" method: "demonstration" verifies: ["REQ-OPS-001"] description: "Injected faults read remotely with their 10 s measurement window" }
test_case "TC-SAF-001" { id: "TC-SAF-001" method: "test" verifies: ["REQ-SAF-001"] description: "Emergency stop under 500 A load: contactor opening and output discharge measured on 50 trips" }
test_case "TC-SAF-002" { id: "TC-SAF-002" method: "test" verifies: ["REQ-SAF-002"] description: "Insulation fault injected before and during charging at the IMD threshold" }
test_case "TC-SAF-003" { id: "TC-SAF-003" method: "test" verifies: ["REQ-SAF-003"] description: "Coolant pump stopped at 500 A: derating at 70 and stop at 90 degrees Celsius on the connector thermocouples" }
test_case "TC-SAF-004" { id: "TC-SAF-004" method: "test" verifies: ["REQ-SAF-004"] description: "Lock actuator inspection; unlock inhibited above 60 V" }
test_case "TC-SEC-001" { id: "TC-SEC-001" method: "inspection" verifies: ["REQ-SEC-001"] description: "PCI DSS assessment of HMI logs and storage: no PAN found" }
test_case "TC-SEC-002" { id: "TC-SEC-002" method: "test" verifies: ["REQ-SEC-002"] description: "Unsigned and wrongly signed images rejected at boot and by UpdateFirmware" }
test_case "TC-SEC-003" { id: "TC-SEC-003" method: "test" verifies: ["REQ-SEC-003"] description: "Altered meter value rejected by the billing service and by the transparency software" }

// ---- Constraints the compiler evaluates -----------------------------------

constraint "Emergency stop within budget" { id: "CST-001" assert: sum("FC-ESTOP", latency) <= "FC-ESTOP".latency_budget }
constraint "Protection trip within budget" { id: "CST-002" assert: sum("FC-TRIP", latency) <= "FC-TRIP".latency_budget }
constraint "Session start within budget" { id: "CST-003" assert: sum("FC-START", latency) <= "FC-START".latency_budget }
constraint "Power cabinet rating covers the connector power" { id: "CST-004" assert: "PN-PWR".voltage * "PN-PWR".rated_current >= "PN-PWR".power }
constraint "Site cap below the grid connection" { id: "CST-005" assert: "LC-SITE".site_power_cap <= "PN-GRID".power }

// ---- Traceability ---------------------------------------------------------

trace "LC-PWR" satisfies "REQ-CHG-001" { rationale: "Seven 50 kW modules, 1000 V / 500 A output" }
trace "LC-PWR" satisfies "REQ-CHG-005" { rationale: "Module efficiency 97 % at rated power" }
trace "LC-15118" satisfies "REQ-CHG-002" { rationale: "Plug-and-charge with the contract certificate" }
trace "LC-HMI" satisfies "REQ-CHG-002" { rationale: "RFID and payment card fallbacks" }
trace "LC-HMI" satisfies "REQ-CHG-003" { rationale: "Certified EMV terminal with pre-authorization" }
trace "LC-CTL" satisfies "REQ-CHG-004" { rationale: "Session sequencing within the FC-START budget" }
trace "LC-PROT" satisfies "REQ-CHG-006" { rationale: "MID DC meter, signed values every 60 s" }
trace "LC-OCPP" satisfies "REQ-CHG-007" { rationale: "OCPP 2.0.1-J client, security profile 3" }
trace "LC-CSMS" satisfies "REQ-CHG-007" { rationale: "OCPP 2.0.1 server with mutual TLS" }
trace "LC-COM" satisfies "REQ-CHG-007" { rationale: "Hosts the protocol stacks and their certificates" }
trace "LC-SITE" satisfies "REQ-CHG-008" { rationale: "600 kW site cap redistributed at 1 Hz" }
trace "LC-MON" satisfies "REQ-OPS-001" { rationale: "Fault records with their measurement window, remote access" }
trace "LC-BACKEND" satisfies "REQ-OPS-001" { rationale: "Operator platform: monitoring and diagnostics" }
trace "LC-BILL" satisfies "REQ-SEC-003" { rationale: "Verifies meter value signatures before invoicing" }
trace "LC-PROT" satisfies "REQ-SEC-003" { rationale: "The meter signs every value" }
trace "LC-CTL" satisfies "REQ-SAF-001" { rationale: "Stop on fault within 40 ms of the trip" }
trace "LC-PROT" satisfies "REQ-SAF-001" { rationale: "Dual-channel emergency stop input, hard-wired trip" }
trace "LC-PROT" satisfies "REQ-SAF-002" { rationale: "IMD on the DC output" }
trace "LC-COOL" satisfies "REQ-SAF-003" { rationale: "Coolant flow controlled on connector temperature" }
trace "LC-PROT" satisfies "REQ-SAF-003" { rationale: "Temperature inputs feed the protection trip" }
trace "LC-CTL" satisfies "REQ-SAF-004" { rationale: "Lock before energizing, unlock below 60 V" }
trace "LC-HMI" satisfies "REQ-SEC-001" { rationale: "Card data stays in the certified terminal" }
trace "LC-CTL" satisfies "REQ-SEC-002" { rationale: "Secure boot and signature check of updates" }
trace "LC-EV" realizes "SA-EV"
trace "SF-AUTH" realizes "OAC-AUTH"
trace "SF-CONV" realizes "OAC-DELIVER"
trace "SF-BAL" realizes "OAC-BALANCE"
trace "SF-REPORT" realizes "OAC-MAINTAIN"
trace "CI-CHG" realizes "PN-CHG"
trace "CI-PWR" realizes "PN-PWR"
trace "CI-DISP" realizes "PN-DISP"
trace "CI-SITEBOX" realizes "PN-SITE"
trace "CI-CSMS" realizes "PN-CLOUD"
trace "CI-FW" realizes "BC-CTL"

// ---- Diagrams -------------------------------------------------------------

diagram "Charger logical architecture" {
  view: lab
  description: "The charger, its protocol stacks and the vehicle"
  shows: ["LC-CTL", "LC-PWR", "LC-PROT", "LC-COOL", "LC-HMI", "LC-OCPP", "LC-15118", "LC-EV"]
}
diagram "Protection path" {
  view: sab
  description: "Emergency stop and protection trip: the functions inside the 100 ms budget"
  shows: ["SF-ESTOP", "SF-MON", "SF-STOP", "SF-REPORT"]
}
diagram "Site deployment" {
  view: pab
  description: "One charger, the site controller, the grid connection and the cloud"
  shows: ["PN-CHG", "PN-PWR", "PN-DISP", "PN-SITE", "PN-CLOUD", "PN-GRID", "PN-EV"]
}
