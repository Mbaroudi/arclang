// Hand-written physics of examples/aerospace/drone_endurance.arc.
// It extends the package `arclang export -f modelica` generates
// (DroneEndurance), which holds the structure and is never edited.
within ;
package DronePhysics "Physics of the drone endurance"
  constant Modelica.Units.SI.Acceleration g = Modelica.Constants.g_n;

  model Pack "Ideal pack at constant voltage; its weight hangs on the frame"
    extends DroneEndurance.Physical.Battery;
    Modelica.Electrical.Analog.Sources.ConstantVoltage cells(V=voltage);
    Modelica.Electrical.Analog.Basic.Ground ground;
    Modelica.Mechanics.Translational.Components.Mass body(m=mass) "The pack itself";
    Modelica.Mechanics.Translational.Sources.Force weight;
    Modelica.Units.SI.Energy drawn(start=0, fixed=true) "Energy drawn, avionics included";
    Real soc(unit="1") "State of charge";
  equation
    connect(cells.p, p);
    connect(cells.n, n);
    connect(ground.p, n);
    der(drawn) = -cells.v * cells.i + avionics_load;
    soc = 1 - drawn / energy;
    connect(mount, body.flange_a);
    connect(body.flange_b, weight.flange);
    weight.f = -mass * g;
    when soc <= 0 then
      terminate("the battery is empty");
    end when;
  end Pack;

  model Rotors "Thrust as commanded; the power momentum theory gives, over the figure of merit and the drive efficiency"
    extends DroneEndurance.Physical.Propulsion;
    parameter Modelica.Units.SI.Area disk = rotors * Modelica.Constants.pi * rotor_radius ^ 2 "Swept area of all rotors";
    Modelica.Units.SI.Density rho "Air density, from the frame's ambient temperature";
    Modelica.Mechanics.Translational.Sources.Force push;
    Modelica.Electrical.Analog.Sources.SignalCurrent load "Current drawn from the pack";
    Modelica.Units.SI.Force thrust "Total thrust";
    Modelica.Units.SI.Power electrical "Power drawn from the pack";
    Modelica.Units.SI.Time elapsed = time "Flight time";
  equation
    thrust = min(max(thrust_cmd, 0), max_thrust);
    push.f = thrust;
    connect(push.flange, lift);
    electrical = thrust * sqrt(thrust / (2 * rho * disk)) / (figure_of_merit * efficiency);
    connect(load.p, p);
    connect(load.n, n);
    load.i = electrical / max(load.v, 1);
  end Rotors;

  model Frame "The frame and its payload: their weight, their altitude"
    extends DroneEndurance.Physical.Airframe;
    parameter Modelica.Units.SI.Density rho = 101325 / (287.05 * ambient) "Air density at the ambient temperature";
    Modelica.Mechanics.Translational.Components.Mass body(m=mass + payload, s(start=0, fixed=true), v(start=0, fixed=true));
    Modelica.Mechanics.Translational.Sources.Force weight;
    Modelica.Units.SI.Length height = body.s "Altitude above the take-off point";
  equation
    connect(lift, body.flange_a);
    connect(body.flange_b, mount);
    connect(weight.flange, body.flange_b);
    weight.f = -(mass + payload) * g;
    altitude = body.s;
  end Frame;

  model AltitudeLoop "Altitude hold: a PID on the altitude, thrust between zero and the limit"
    extends DroneEndurance.Behavior.FlightController.AltitudeHold;
    Modelica.Blocks.Sources.Constant setpoint(k=target_altitude);
    Modelica.Blocks.Continuous.LimPID pid(controllerType=Modelica.Blocks.Types.SimpleController.PID, k=12, Ti=6, Td=1.5, yMax=250, yMin=0, initType=Modelica.Blocks.Types.Init.InitialOutput, y_start=0);
  equation
    connect(setpoint.y, pid.u_s);
    connect(altitude, pid.u_m);
    connect(pid.y, thrust_cmd);
  end AltitudeLoop;

  model Controller "The flight controller, its altitude hold implemented"
    extends DroneEndurance.Physical.FlightController(redeclare DronePhysics.AltitudeLoop altitudeHold);
  end Controller;

  model SurveyHover "Configuration Survey: climb to 50 m and hover"
    extends DroneEndurance.Systems.Survey(
      redeclare DronePhysics.Pack battery,
      redeclare DronePhysics.Rotors propulsion(rho=airframe.rho),
      redeclare DronePhysics.Frame airframe,
      redeclare DronePhysics.Controller flightController);
  end SurveyHover;

  model LongHover "Configuration Long endurance: climb to 50 m and hover"
    extends DroneEndurance.Systems.LongEndurance(
      redeclare DronePhysics.Pack battery,
      redeclare DronePhysics.Rotors propulsion(rho=airframe.rho),
      redeclare DronePhysics.Frame airframe,
      redeclare DronePhysics.Controller flightController);
  end LongHover;
  annotation(uses(Modelica(version="4.1.0")));
end DronePhysics;
