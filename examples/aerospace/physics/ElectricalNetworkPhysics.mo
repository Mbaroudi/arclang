// Hand-written physics of examples/aerospace/aircraft_electrical_network.arc.
// It extends the package `arclang export -f modelica` generates
// (AircraftElectricalNetwork), which holds the structure and is never edited.
within ;
package ElectricalNetworkPhysics "Physics of the aircraft electrical network"
  model Generator "A regulated source behind its line resistance, disconnected when it fails"
    extends AircraftElectricalNetwork.Physical.StarterGenerator;
    Modelica.Electrical.Analog.Sources.ConstantVoltage regulated(V=voltage);
    Modelica.Electrical.Analog.Basic.Resistor line(R=internal_resistance);
    Modelica.Electrical.Analog.Ideal.IdealOpeningSwitch contactor "Opens when the generator fails";
    Boolean failed = time >= fails_at;
  equation
    contactor.control = failed;
    connect(regulated.p, line.p);
    connect(line.n, contactor.p);
    connect(contactor.n, p);
    connect(regulated.n, n);
    online = if failed then 0 else 1;
  end Generator;

  model Battery "Open-circuit voltage falling with the charge; less capacity and more resistance in the cold"
    extends AircraftElectricalNetwork.Physical.MainBattery;
    parameter Real capacity_factor(unit="1") = 1 - 0.006 * (293.15 - temperature) "Usable fraction of the capacity at this temperature";
    parameter Real resistance_factor(unit="1") = 1 + 0.02 * max(0, 293.15 - temperature) "Internal resistance rise in the cold";
    Modelica.Electrical.Analog.Sources.SignalVoltage ocv "Open-circuit voltage";
    Modelica.Electrical.Analog.Basic.Resistor resistance(R=internal_resistance * resistance_factor);
    Modelica.Electrical.Analog.Basic.Ground ground;
    Real soc(unit="1", start=initial_soc, fixed=true) "State of charge";
    Modelica.Units.SI.Current current = -resistance.i "Discharge current";
  equation
    ocv.v = empty_voltage + (full_voltage - empty_voltage) * soc;
    connect(p, resistance.p);
    connect(resistance.n, ocv.p);
    connect(ocv.n, n);
    connect(ground.p, n);
    der(soc) = resistance.i / (capacity * capacity_factor);
  end Battery;

  model Essential "Constant-power loads: avionics, flight instruments, radios"
    extends AircraftElectricalNetwork.Physical.EssentialLoads;
    parameter Modelica.Units.SI.Voltage dropout = 18 "Below this the power supplies drop out";
    Modelica.Electrical.Analog.Sources.SignalCurrent load;
    Modelica.Units.SI.Voltage voltage = p.v - n.v "Essential bus voltage";
  equation
    connect(load.p, p);
    connect(load.n, n);
    // Constant power down to the dropout voltage, then resistive: the bus sags, it never jumps.
    load.i = if voltage > dropout then power / voltage else power * voltage / dropout ^ 2;
    // The avionics drop out: the bus has collapsed, the run ends there.
    when voltage < dropout then
      terminate("the essential bus collapsed");
    end when;
  end Essential;

  model Cabin "Constant-power loads the relays cut when shed"
    extends AircraftElectricalNetwork.Physical.NonEssentialLoads;
    Modelica.Electrical.Analog.Sources.SignalCurrent load;
    Modelica.Units.SI.Voltage voltage = p.v - n.v;
    Modelica.Units.SI.Power drawn = power * (1 - shed) "Power drawn";
  equation
    connect(load.p, p);
    connect(load.n, n);
    load.i = drawn / max(voltage, 1);
  end Cabin;

  model Shedding "Shed the cabin as soon as the generator is offline"
    extends AircraftElectricalNetwork.Behaviour.BusPowerControlUnit.SheddingLogic;
  equation
    shed = 1 - online;
  end Shedding;

  model ControlUnit "The bus power control unit, its shedding logic implemented"
    extends AircraftElectricalNetwork.Physical.BusPowerControlUnit(redeclare ElectricalNetworkPhysics.Shedding sheddingLogic);
  end ControlUnit;

  model LightBattery "Configuration Light battery: the generator fails at 10 min"
    extends AircraftElectricalNetwork.Systems.LightBattery(
      redeclare ElectricalNetworkPhysics.Generator starterGenerator,
      redeclare ElectricalNetworkPhysics.Battery mainBattery,
      redeclare ElectricalNetworkPhysics.Essential essentialLoads,
      redeclare ElectricalNetworkPhysics.Cabin nonEssentialLoads,
      redeclare ElectricalNetworkPhysics.ControlUnit busPowerControlUnit);
  end LightBattery;

  model HeavyBattery "Configuration Heavy battery: the generator fails at 10 min"
    extends AircraftElectricalNetwork.Systems.HeavyBattery(
      redeclare ElectricalNetworkPhysics.Generator starterGenerator,
      redeclare ElectricalNetworkPhysics.Battery mainBattery,
      redeclare ElectricalNetworkPhysics.Essential essentialLoads,
      redeclare ElectricalNetworkPhysics.Cabin nonEssentialLoads,
      redeclare ElectricalNetworkPhysics.ControlUnit busPowerControlUnit);
  end HeavyBattery;
  annotation(uses(Modelica(version="4.1.0")));
end ElectricalNetworkPhysics;
