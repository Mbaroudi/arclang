// Hybrid powertrain, P1 configuration: the case study of Banagar, Kramsu,
// Könnö and Andwari, "Connecting systems engineering and domain expertise
// through a toolchain: A simulation-integrated MBSE approach", Journal of
// Systems and Software 232 (2026) 112671, doi:10.1016/j.jss.2025.112671,
// CC BY 4.0. Their SysML v2 (appendix B, Fig. B.4) written in ArcLang: the
// same five parts, ports and eleven interfaces, with the units their
// attributes leave implicit.
//
// P1: the electric motor sits on the crankshaft of the combustion engine;
// a 48 V battery of 5 kWh.
//
//   arclang check examples/automotive/hybrid-powertrain/hybrid_powertrain_p1.arc
//   arclang diagram examples/automotive/hybrid-powertrain/hybrid_powertrain_p1.arc --view la

model HybridPowertrainP1 {
    name: "Hybrid powertrain, P1"
    version: "1.0"
}

requirements {
    requirement "SYS-HP-001" {
        description: "Every interface between two subsystems carries its signal from the subsystem that produces it to the one that consumes it"
        priority: "High"
    }
    requirement "SYS-HP-002" {
        description: "Every attribute of a subsystem used as a simulation parameter states its unit"
        priority: "Medium"
    }
}

logical_architecture "P1 hybrid powertrain" {
    component "vehicle_p1" {
        id: "vehicle_p1"
        mass: 1500 kg
        vehicle_class: "A"
        port in "driveline_trq_in"
        port out "driveline_speed_out"
        port out "velocity_out"
        port out "load_out"
    }
    component "engine_p1" {
        id: "engine_p1"
        max_power: 100 kW
        max_torque_nm: 200  // N·m: ArcLang has no torque unit yet
        port out "engine_trq_out"
        port in "engine_speed_in"
        port in "em_trq_in"
        port out "em_spe_out"
        port in "eng_ld_sgn_in"
        port out "eng_fuel_consumption"
    }
    component "em" {
        id: "em"
        max_power: 15 kW
        max_torque_nm: 50  // N·m: ArcLang has no torque unit yet
        port out "mtr_trq_out"
        port in "mtr_speed_in"
        port in "vol_in"
        port out "cur_out"
        port in "mtr_ld_sgn_in"
    }
    component "bat" {
        id: "bat"
        capacity: 5 kWh
        max_voltage: 48 V
        port out "vol_out"
        port in "cur_in"
        port out "soc_out"
    }
    component "ctrl" {
        id: "ctrl"
        port out "en_ld_sgn_out"
        port out "mtr_ld_sgn_out"
        port in "vehicle_vel"
        port in "soc_in"
        port in "load_in"
    }

    component_exchange "int1: trq_int" { from_port: "engine_p1.engine_trq_out" to_port: "vehicle_p1.driveline_trq_in" }
    component_exchange "int2: spe_int" { from_port: "vehicle_p1.driveline_speed_out" to_port: "engine_p1.engine_speed_in" }
    component_exchange "int3: mon_vel_int" { from_port: "vehicle_p1.velocity_out" to_port: "ctrl.vehicle_vel" }
    component_exchange "int4: ld_sgn_int" { from_port: "vehicle_p1.load_out" to_port: "ctrl.load_in" }
    component_exchange "int5: trq_int" { from_port: "em.mtr_trq_out" to_port: "engine_p1.em_trq_in" }
    component_exchange "int6: spe_int" { from_port: "engine_p1.em_spe_out" to_port: "em.mtr_speed_in" }
    component_exchange "int7: ld_sgn_int" { from_port: "ctrl.en_ld_sgn_out" to_port: "engine_p1.eng_ld_sgn_in" }
    component_exchange "int8: vol_int" { from_port: "bat.vol_out" to_port: "em.vol_in" }
    component_exchange "int9: cur_int" { from_port: "em.cur_out" to_port: "bat.cur_in" }
    component_exchange "int10: ld_sgn_int" { from_port: "ctrl.mtr_ld_sgn_out" to_port: "em.mtr_ld_sgn_in" }
    component_exchange "int11: mon_soc_int" { from_port: "bat.soc_out" to_port: "ctrl.soc_in" }
}

trace "vehicle_p1" satisfies "SYS-HP-001" {}
trace "engine_p1" satisfies "SYS-HP-001" {}
trace "em" satisfies "SYS-HP-001" {}
trace "bat" satisfies "SYS-HP-002" {}
trace "ctrl" satisfies "SYS-HP-001" {}
