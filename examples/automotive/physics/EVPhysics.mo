// Hand-written physics of examples/automotive/ev_range_simulation.arc.
// It extends the package `arclang export -f modelica` generates
// (EVRangeSimulation), which holds the structure and is never edited:
// each model below extends a generated partial model and adds equations,
// and the two systems redeclare every part of a generated configuration.
within ;
package EVPhysics "Physics of the EV range simulation"
  model Battery "Ideal pack at constant voltage; state of charge from the energy drawn"
    extends EVRangeSimulation.Physical.BatteryPack;
    Modelica.Electrical.Analog.Sources.ConstantVoltage cells(V=voltage) "Open-circuit voltage";
    Modelica.Electrical.Analog.Basic.Ground ground;
    Modelica.Units.SI.Power power "Power delivered at the terminals";
    Modelica.Units.SI.Energy drawn(start=0, fixed=true) "Energy drawn, auxiliaries included";
    Real soc(unit="1") "State of charge";
  equation
    connect(cells.p, hv_p);
    connect(cells.n, hv_n);
    connect(ground.p, hv_n);
    power = -cells.v * cells.i;
    der(drawn) = power + auxiliary_load;
    soc = 1 - drawn / energy;
    // An empty pack stops the vehicle: the trip ends there, short of its length.
    when soc <= 0 then
      terminate("the battery is empty");
    end when;
  end Battery;

  model Drive "Torque-controlled motor and reduction gear, drawing its power from the HV bus"
    extends EVRangeSimulation.Physical.DriveUnit;
    Modelica.Mechanics.Rotational.Sources.Torque motor;
    Modelica.Mechanics.Rotational.Components.Inertia rotor(J=0.05);
    Modelica.Mechanics.Rotational.Components.IdealGear gear(ratio=gear_ratio);
    Modelica.Electrical.Analog.Sources.SignalCurrent load "Current drawn from the bus";
    Modelica.Units.SI.Power mechanical "Shaft power of the motor";
    Modelica.Units.SI.Power electrical "Power drawn from the bus";
  equation
    connect(torque_cmd, motor.tau);
    connect(motor.flange, rotor.flange_a);
    connect(rotor.flange_b, gear.flange_a);
    connect(gear.flange_b, shaft);
    connect(load.p, hv_p);
    connect(load.n, hv_n);
    mechanical = motor.tau * rotor.w;
    electrical = noEvent(if mechanical >= 0 then mechanical / efficiency else mechanical * efficiency);
    load.i = electrical / max(load.v, 1);
  end Drive;

  model Body "Longitudinal dynamics: mass, aerodynamic drag and rolling resistance"
    extends EVRangeSimulation.Physical.VehicleBody;
    constant Modelica.Units.SI.Acceleration g = Modelica.Constants.g_n;
    parameter Modelica.Units.SI.Density rho = 101325 / (287.05 * ambient) "Air density at the ambient temperature";
    Modelica.Mechanics.Rotational.Components.IdealRollingWheel tyres(radius=wheel_radius);
    Modelica.Mechanics.Translational.Components.Mass car(m=mass, s(start=0, fixed=true), v(start=0, fixed=true));
    Modelica.Mechanics.Translational.Sources.Force resistance;
    Modelica.Units.SI.Length distance = car.s "Distance driven";
  equation
    connect(wheel, tyres.flangeR);
    connect(tyres.flangeT, car.flange_a);
    connect(car.flange_b, resistance.flange);
    resistance.f = -(0.5 * rho * drag_area * car.v * abs(car.v) + rolling_resistance * mass * g * tanh(car.v / 0.5));
    speed = car.v;
  end Body;

  model SpeedControl "Cruise control: a PI on the vehicle speed, the motor torque limited"
    extends EVRangeSimulation.Behaviour.VehicleControlUnit.SpeedController;
    parameter Modelica.Units.SI.Torque tauMax = 350 "Motor torque limit";
    Modelica.Blocks.Sources.Constant setpoint(k=target_speed);
    Modelica.Blocks.Continuous.LimPID pi(controllerType=Modelica.Blocks.Types.SimpleController.PI, k=200, Ti=10, yMax=tauMax, yMin=-tauMax);
  equation
    connect(setpoint.y, pi.u_s);
    connect(speed, pi.u_m);
    connect(pi.y, torque_cmd);
  end SpeedControl;

  model ControlUnit "The vehicle control unit, its speed controller implemented"
    extends EVRangeSimulation.Physical.VehicleControlUnit(redeclare EVPhysics.SpeedControl speedController);
  end ControlUnit;

  model CityRange "Configuration City (standard pack) on the winter highway"
    extends EVRangeSimulation.Systems.City(
      redeclare EVPhysics.Battery batteryPack,
      redeclare EVPhysics.Drive driveUnit,
      redeclare EVPhysics.Body vehicleBody,
      redeclare EVPhysics.ControlUnit vehicleControlUnit);
    parameter Modelica.Units.SI.Length trip = 300e3 "Length of the trip: the simulation ends there";
  equation
    when vehicleBody.distance >= trip then
      terminate("the trip is driven");
    end when;
  end CityRange;

  model TouringRange "Configuration Touring (long range pack) on the winter highway"
    extends EVRangeSimulation.Systems.Touring(
      redeclare EVPhysics.Battery batteryPack,
      redeclare EVPhysics.Drive driveUnit,
      redeclare EVPhysics.Body vehicleBody,
      redeclare EVPhysics.ControlUnit vehicleControlUnit);
    parameter Modelica.Units.SI.Length trip = 300e3 "Length of the trip: the simulation ends there";
  equation
    when vehicleBody.distance >= trip then
      terminate("the trip is driven");
    end when;
  end TouringRange;
  annotation(uses(Modelica(version="4.1.0")));
end EVPhysics;
