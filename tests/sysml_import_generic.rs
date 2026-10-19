//! What the SysML v2 import reads of a model written without the Arcadia
//! profile, in the style of the OMG examples: bindings, performed actions,
//! flows between their parameters, and messages between lifelines. The
//! shapes are those of the Simple Vehicle Model (SysML v2 spec, annex A).

use arclang::compiler::sysml_import;

fn imported(text: &str) -> sysml_import::Imported {
    sysml_import::import_text(text).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.rejected.unwrap_or_default()))
}

fn assert_has(source: &str, expected: &str) {
    assert!(source.contains(expected), "expected `{expected}` in:\n{source}");
}

/// A vehicle whose parts are joined by a connection and a binding, and
/// perform actions of an action tree that flow into each other.
const VEHICLE: &str = "package Vehicle {
    port def TorquePort;
    action def GenerateTorque { out torque; }
    action def AmplifyTorque { in torque; }
    package ActionTree {
        action providePower {
            action generateTorque : GenerateTorque;
            action amplifyTorque : AmplifyTorque;
            flow generateTorque.torque to amplifyTorque.torque;
        }
    }
    part vehicle {
        port drivePort : TorquePort;
        perform ActionTree::providePower;
        part engine {
            port drivePwrPort : TorquePort;
            port shaftPort : TorquePort;
            perform ActionTree::providePower.generateTorque;
        }
        part transmission {
            port clutchPort : TorquePort;
            perform ActionTree::providePower.amplifyTorque;
        }
        connect engine.drivePwrPort to transmission.clutchPort;
        bind drivePort = engine.shaftPort;
    }
}
";

#[test]
fn a_binding_is_an_exchange_between_the_ports_it_binds() {
    let source = imported(VEHICLE).source;
    assert_has(&source, "from_port: \"engine.drivePwrPort\"\n        to_port: \"transmission.clutchPort\"");
    assert_has(&source, "from_port: \"vehicle.drivePort\"\n        to_port: \"engine.shaftPort\"");
}

#[test]
fn performed_actions_are_functions_written_in_the_parts_that_perform_them() {
    let source = imported(VEHICLE).source;
    // Everything sits at the level of the parts: no system analysis of loose
    // functions. A function used once is named by its definition.
    assert!(!source.contains("system_analysis"), "{source}");
    // Each leaf is written in the part that performs it, as Capella
    // allocates it; the action it decomposes stays at the level.
    assert_has(&source, "component \"engine\" {\n            id: \"engine\"\n            port inout \"drivePwrPort\"\n            port inout \"shaftPort\"\n            function \"GenerateTorque\" {\n                id: \"GenerateTorque\"\n                parent: \"providePower\"");
    assert_has(&source, "function \"AmplifyTorque\" {\n                id: \"AmplifyTorque\"\n                parent: \"providePower\"");
    // The flow between the parameters of the performed actions.
    assert_has(&source, "functional_exchange");
    assert_has(&source, "from: \"GenerateTorque.torque\"\n        to: \"AmplifyTorque.torque\"");
}

#[test]
fn messages_between_parts_that_perform_actions_are_a_scenario() {
    let text = "package Seq {
    item def IgnitionCmd;
    part def Driver;
    part def Car;
    part part0 {
        action startVehicle {
            action turnVehicleOn;
            action trigger1;
        }
        part driver : Driver { perform startVehicle.turnVehicleOn; }
        part car : Car { perform startVehicle.trigger1; }
        message of ignitionCmd : IgnitionCmd from driver.turnVehicleOn to car.trigger1;
    }
}
";
    let source = imported(text).source;
    // `driver : Driver`, the one usage of its definition, is named by it.
    assert_has(&source, "scenario \"part0 interaction\"");
    assert_has(&source, "participants: [\"Driver\", \"Car\"]");
    assert_has(&source, "message \"Driver\" -> \"Car\" \"IgnitionCmd\"");
}

#[test]
fn messages_between_events_on_ports_of_nested_parts_are_a_scenario() {
    let text = "package Cruise {
    item def SensedSpeed;
    part vehicle {
        part speedSensor { port speedSensorPort; }
        part software { part cruiseController { port speedPort; } }
        connect speedSensor.speedSensorPort to software.cruiseController.speedPort;
    }
    occurrence CruiseControl {
        part vehicle_b :> vehicle {
            part redefines speedSensor { port redefines speedSensorPort { event occurrence sent; } }
            part redefines software { part redefines cruiseController { port redefines speedPort { event occurrence received; } } }
            message sendSensedSpeed of SensedSpeed from speedSensor.speedSensorPort.sent to software.cruiseController.speedPort.received;
        }
    }
}
";
    let source = imported(text).source;
    assert_has(&source, "scenario \"CruiseControl\"");
    assert_has(&source, "participants: [\"speedSensor\", \"cruiseController\"]");
    assert_has(&source, "message \"speedSensor\" -> \"cruiseController\" \"sendSensedSpeed\"");
}

#[test]
fn a_connector_left_out_is_counted() {
    let text = "package P {
    part a { port p; }
    connect a.p to nowhere.q;
}
";
    let counts = imported(text).counts;
    let left_out: usize = counts.iter().filter(|(k, _)| k.contains("connector") && k.contains("left out")).map(|(_, v)| v).sum();
    assert_eq!(left_out, 1, "{counts:?}");
}

#[test]
fn each_configuration_and_action_tree_gets_its_own_view() {
    let source = imported(VEHICLE).source;
    // The interconnection view of the part, as a SysML tool draws it, and
    // the dataflow of the action tree over the parts that perform it.
    assert_has(&source, "diagram \"vehicle interconnection\" {\n    view: lab\n    shows: [\"vehicle\"]\n}");
    assert_has(&source, "diagram \"providePower dataflow\" {\n    view: lab\n    shows: [\"GenerateTorque\", \"AmplifyTorque\"]\n}");
    let result = arclang::compiler::Compiler::new(arclang::compiler::CompilerConfig::default()).compile_string(&source).expect("compiles");
    let diagrams = arclang::compiler::diagram::build_diagrams(&result.ast);
    let titles: Vec<&str> = diagrams.diagrams.iter().map(|d| d.title.as_str()).collect();
    assert!(titles.contains(&"vehicle interconnection") && titles.contains(&"providePower dataflow"), "{titles:?}\n{:?}", diagrams.diagnostics);
}

#[test]
fn a_connection_end_is_the_port_of_the_part_it_names_not_of_its_definition() {
    // Two configurations of one definition: each connection joins the ports
    // of its own parts, as written, whatever part first carried the ports.
    let text = "package P {
    port def FuelPort;
    part def Tank { port outlet : FuelPort; }
    part def Engine { port inlet : FuelPort; }
    part car_a { part tank : Tank; part engine : Engine; connect tank.outlet to engine.inlet; }
    part car_b { part tank : Tank; part engine : Engine; connect tank.outlet to engine.inlet; }
    part context { part car_c :> car_b; part pump { port nozzle : FuelPort; } connect pump.nozzle to car_c.tank.outlet; }
    part def Car { port cap : FuelPort; }
    part car_d : Car { part tank : Tank; bind tank.outlet = cap; }
    part car_e : Car;
    part car_f :> car_d;
}
";
    let source = imported(text).source;
    assert_has(&source, "from_port: \"car_a.tank.outlet\"\n        to_port: \"car_a.engine.inlet\"");
    // The second `tank` has a distinct id; its parent qualifies the path.
    assert_has(&source, "from_port: \"car_b.tank_.outlet\"\n        to_port: \"car_b.engine_.inlet\"");
    // A port named alone is one of the part holding the binding, inherited or not.
    assert_has(&source, "to_port: \"car_d.cap\"");
    // A part subsetting another is of its type, and has its ports.
    assert_has(&source, "component \"car_f\" {\n        id: \"car_f\"\n        is: \"Car\"");
}

#[test]
fn lifelines_of_parts_sharing_a_name_are_their_ids() {
    // Two configurations both have an `engine`: the scenario names the
    // lifeline by its id, and the imported model compiles (the import fails otherwise).
    let text = "package P {
    part car_a { part engine { port p; } part ecu { port q; } }
    part car_b { part engine { port p { event occurrence got; } } part ecu { port q { event occurrence sent; } } }
    occurrence Start { part b :> car_b { message go from ecu.q.sent to engine.p.got; } }
}
";
    let source = imported(text).source;
    assert_has(&source, "scenario \"Start\"");
    assert_has(&source, "message \"ecu_\" -> \"engine_\" \"go\"");
}

#[test]
fn a_flow_between_items_of_ports_is_an_exchange_between_the_ports() {
    let text = "package Rooms {
    item def Air;
    port def Door { out item air : Air; }
    part def Hall { port outlet : Door; }
    part def Room { port inlet : ~Door; }
    part school { part h : Hall; part c : Room; flow HallToRoom from h.outlet.air to c.inlet.air; }
}
";
    let source = imported(text).source;
    // `h`, the one usage of `Hall`, is named by it; the item is carried.
    assert_has(&source, "component_exchange \"HallToRoom\" {\n        id: \"HallToRoom\"\n        from_port: \"Hall.outlet\"\n        to_port: \"Room.inlet\"\n        exchange_items: [\"Air\"]");
}

#[test]
fn bindings_between_values_are_not_counted_as_lost_connections() {
    let text = "package C {
    constraint def Sum { in a : Real; in b : Real; a + b > 0 }
    part p { attribute x : Real; attribute y : Real; constraint s : Sum { bind a = x; bind b = y; } }
}
";
    let counts = imported(text).counts;
    assert!(!counts.keys().any(|k| k.starts_with("connectors")), "{counts:?}");
    assert_eq!(counts.get("bindings between values (not architecture, left out)"), Some(&2), "{counts:?}");
}

#[test]
fn accept_and_send_actions_are_functions() {
    let text = "package Start {
    item def IgnitionCmd;
    part car {
        port ignitionPort;
        action startVehicle {
            action trigger accept ignitionCmd : IgnitionCmd via ignitionPort;
            flow of IgnitionCmd from trigger.ignitionCmd to startEngine.ignitionCmd;
            action startEngine { in item ignitionCmd : IgnitionCmd; }
            action report send ignitionCmd via ignitionPort;
        }
    }
}
";
    let source = imported(text).source;
    assert_has(&source, "function \"trigger\"");
    assert_has(&source, "function \"report\"");
    assert_has(&source, "from: \"trigger.ignitionCmd\"\n        to: \"startEngine.ignitionCmd\"");
}

#[test]
fn a_formula_is_not_written_as_the_quantity_its_key_holds() {
    let text = "package M { part car { attribute dryMass = 1000 [kg]; attribute cargoMass = 200 [kg]; attribute mass = dryMass + cargoMass; } }";
    let imported = imported(text);
    assert_has(&imported.source, "mass_expression: \"dryMass + cargoMass\"");
    assert!(!imported.notes.iter().any(|n| n.contains("is not a quantity")), "{:?}", imported.notes);
}

#[test]
fn the_entry_transition_of_a_region_is_its_initial_state() {
    let text = "package S {
    part car {
        exhibit state carStates parallel {
            state operating {
                entry action initial;
                state off;
                state on;
                transition initial then off;
                transition off_to_on first off then on;
            }
            state health {
                entry; then normal;
                state normal;
            }
        }
    }
}
";
    let imported = imported(text);
    assert_has(&imported.source, "state \"on\" { id: \"on\" }\n        initial: \"off\"");
    assert_has(&imported.source, "state \"normal\" { id: \"normal\" }\n        initial: \"normal\"");
    assert!(!imported.counts.contains_key("transitions to states not written (left out)"), "{:?}", imported.counts);
}
