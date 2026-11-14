// Hybrid powertrain, P4 configuration: the case study of Banagar, Kramsu,
// Könnö and Andwari, "Connecting systems engineering and domain expertise
// through a toolchain: A simulation-integrated MBSE approach", Journal of
// Systems and Software 232 (2026) 112671, doi:10.1016/j.jss.2025.112671,
// CC BY 4.0. Their SysML v2 (appendix B, Fig. B.5) written in ArcLang, as
// published.
//
// P4: the electric motor drives the second axle on its own; a 400 V battery
// of 10 kWh.
//
// As printed, the motor's two mechanical ports are the other way round from
// the P1 configuration (`mtr_trq_out : ~mech_trq`, `mtr_speed_in :
// mech_spe`): its torque "output" is an input and its speed "input" an
// output. Written with those directions, int5 joins two inputs and int6 two
// outputs, and `arclang check` says so (rule interface.direction). The
// corrected model is hybrid_powertrain_p4_fixed.arc.
//
//   arclang check examples/automotive/hybrid-powertrain/hybrid_powertrain_p4.arc

model HybridPowertrainP4 {
    name: "Hybrid powertrain, P4 (as published)"
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

logical_architecture "P4 hybrid powertrain" {
    component "vehicle_p4" {
        id: "vehicle_p4"
        mass: 1500 kg
        vehicle_class: "A"
        port in "driveline_trq_in_1"
        port out "driveline_speed_out_1"
        port in "driveline_trq_in_2"
        port out "driveline_speed_out_2"
        port out "velocity_out"
        port out "load_out"
    }
    component "engine_p4" {
        id: "engine_p4"
        max_power: 100 kW
        max_torque_nm: 200  // N·m: ArcLang has no torque unit yet
        port out "engine_trq_out"
        port in "engine_speed_in"
        port in "eng_ld_sgn_in"
        port out "eng_fuel_consumption"
    }
    component "em" {
        id: "em"
        max_power: 30 kW
        max_torque_nm: 100  // N·m: ArcLang has no torque unit yet
        // As published (Fig. B.5): `port mtr_trq_out:~mech_trq;` and
        // `port mtr_speed_in:mech_spe;`, the reverse of the P1 motor.
        port in "mtr_trq_out"
        port out "mtr_speed_in"
        port in "vol_in"
        port out "cur_out"
        port in "mtr_ld_sgn_in"
    }
    component "bat" {
        id: "bat"
        capacity: 10 kWh
        max_voltage: 400 V
        port out "vol_out"
        port in "cur_in"
        port out "soc_out"
    }
    component "ctrl" {
        id: "ctrl"
        port out "en_ld_sgn_out"
        port out "mtr_ld_sgn_out"
        port in "vehicle_vel"
        port in "load_in"
    }

    component_exchange "int1: trq_int" { from_port: "engine_p4.engine_trq_out" to_port: "vehicle_p4.driveline_trq_in_1" }
    component_exchange "int2: spe_int" { from_port: "vehicle_p4.driveline_speed_out_1" to_port: "engine_p4.engine_speed_in" }
    component_exchange "int3: mon_vel_int" { from_port: "vehicle_p4.velocity_out" to_port: "ctrl.vehicle_vel" }
    component_exchange "int4: ld_sgn_int" { from_port: "vehicle_p4.load_out" to_port: "ctrl.load_in" }
    component_exchange "int5: trq_int" { from_port: "em.mtr_trq_out" to_port: "vehicle_p4.driveline_trq_in_2" }
    component_exchange "int6: spe_int" { from_port: "vehicle_p4.driveline_speed_out_2" to_port: "em.mtr_speed_in" }
    component_exchange "int7: ld_sgn_int" { from_port: "ctrl.en_ld_sgn_out" to_port: "engine_p4.eng_ld_sgn_in" }
    component_exchange "int8: vol_int" { from_port: "bat.vol_out" to_port: "em.vol_in" }
    component_exchange "int9: cur_int" { from_port: "em.cur_out" to_port: "bat.cur_in" }
    component_exchange "int10: ld_sgn_int" { from_port: "ctrl.mtr_ld_sgn_out" to_port: "em.mtr_ld_sgn_in" }
}

trace "vehicle_p4" satisfies "SYS-HP-001" {}
trace "engine_p4" satisfies "SYS-HP-001" {}
trace "em" satisfies "SYS-HP-001" {}
trace "bat" satisfies "SYS-HP-002" {}
trace "ctrl" satisfies "SYS-HP-001" {}
